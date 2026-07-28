# Plantillas de Contratos Soroban

[![CI](https://github.com/Fidelis900/soroban-starter-kit/actions/workflows/ci.yml/badge.svg)](https://github.com/Fidelis900/soroban-starter-kit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Fidelis900/soroban-starter-kit/branch/main/graph/badge.svg)](https://codecov.io/gh/Fidelis900/soroban-starter-kit)

Una colección curada de plantillas de contratos inteligentes Soroban listos para producción. Estas plantillas ayudan a los desarrolladores a iniciar rápidamente casos de uso comunes en Soroban (la plataforma de contratos inteligentes de Stellar) para DeFi, pagos, gobernanza y más.

> **English version**: [README.md](README.md)
> **Disponible en français**: [README.fr.md](README.fr.md)

## 🚀 Inicio Rápido

```bash
# Clonar el repositorio
git clone https://github.com/your-username/soroban-contract-templates.git
cd soroban-contract-templates

# Construir todos los contratos
make build

# Ejecutar pruebas
make test

# Desplegar en testnet
make deploy-testnet

# Ver todos los comandos disponibles
make help
```

O usa `just` (consulta [dev-environment.md](docs/dev-environment.md) para la instalación):

```bash
just build
just test
just deploy-testnet
just --list
```

## 📦 Plantillas de Contratos

| Plantilla | Descripción | Casos de Uso | Estado |
|----------|-------------|-----------|---------|
| **Token** | Token fungible personalizado con controles mint/burn/admin | Tokens DeFi, tokens de gobernanza, tokens de utilidad | ✅ Completo |
| **Escrow** | Depósito en garantía de dos partes con mecanismo de timeout y reembolso | Comercio P2P, pagos por servicio, pagos por hitos | ✅ Completo |
| **Vesting** | Liberación de tokens con acantilado + cronograma lineal | Asignaciones de equipo, bloqueos de inversores, subvenciones de empleados | ✅ Completo |
| **Staking** | Staking de tokens con distribución proporcional de recompensas | DeFi yield, incentivos de protocolo, minería de liquidez | ✅ Completo |
| **Multisig** | Cartera N-de-M para llamadas de contratos aprobadas por umbral | Tesorerías DAO, carteras de equipo, administración compartida | ✅ Completo |
| **Subscription** | Contrato de pago recurrente con extracción de tokens | Facturación SaaS, pagos en streaming, cuotas de membresía | ✅ Completo |
| **Timelock** | Liberación de tokens bloqueada en el tiempo a un beneficiario | Bloqueos de tokens de equipo, pagos retrasados, timelocks de gobernanza | ✅ Completo |
| **NFT** | Token no fungible con minting administrativo y límite de suministro opcional | Coleccionables digitales, propiedad en cadena, tokens de acceso | ✅ Completo |
| **DAO** | Gobernanza en cadena con votación ponderada por tokens | Actualizaciones de protocolo, gestión de tesorería, decisiones comunitarias | ✅ Completo |
| **Swap** | Intercambio atómico de dos partes con plazo límite | Intercambio de tokens P2P, operaciones OTC, swaps DeFi sin confianza | ✅ Completo |
| **Oracle** | Consumidor de feed de precios con validación de antigüedad | Feeds de precios DeFi, consumo de datos en cadena, verificaciones de actualización | ✅ Completo |
| **Lottery** | Lotería verificable en cadena con aleatoriedad commit-reveal | Sorteos, sorteos justos, distribución descentralizada de premios | ✅ Completo |

### Características del Contrato Token
- **Interfaz Estándar**: Compatible completo con tokens de Soroban
- **Controles Administrativos**: Mint, burn y gestión de admin
- **Soporte de Metadatos**: Nombre, símbolo y decimales
- **Sistema de Asignación**: Funcionalidad approve y transfer_from
- **Emisión de Eventos**: Todas las operaciones emiten eventos para seguimiento
- **Manejo de Errores**: Tipos de error personalizados para mejor depuración

### Características del Contrato Escrow
- **Seguridad de Dos Partes**: Transacciones seguras comprador-vendedor
- **Protección de Plazo**: Reembolsos automáticos después del plazo
- **Soporte de Árbitro**: Resolución de disputas por terceros
- **Gestión de Estado**: Ciclo de vida de transacción claro
- **Agnóstico de Token**: Funciona con cualquier token Soroban
- **Emisión de Eventos**: Todas las operaciones emiten eventos para seguimiento

### Características del Contrato Vesting
- **Cronograma Acantilado + Lineal**: Tokens desbloqueados linealmente entre `cliff_ledger` y `end_ledger`
- **Revocación por Admin**: El admin puede cancelar tokens no adquiridos en cualquier momento; los tokens adquiridos permanecen reclamables
- **Reclamaciones Incrementales**: El beneficiario reclama tokens acumulados bajo demanda
- **Agnóstico de Token**: Funciona con cualquier token compatible con Soroban
- **Emisión de Eventos**: Eventos `initialized`, `claimed` y `revoked` para seguimiento fuera de cadena
- **Gestión de TTL**: La TTL de almacenamiento de instancia se extiende en cada interacción

### Características del Contrato Staking
- **Recompensas Proporcionales**: Las recompensas se distribuyen pro-rata según la participación de cada staker
- **Acumulador de Recompensa por Token**: Patrón de acumulador global eficiente en gas; sin bucles por staker
- **Tokens de Stake y Recompensa Separados**: El token de stake y el token de recompensa pueden ser iguales o diferentes
- **Depósitos de Recompensa por Admin**: El admin llama a `add_rewards` para aumentar el fondo de recompensas en cualquier momento
- **Reclamaciones Incrementales**: Los stakers llaman a `claim_rewards` independientemente; las recompensas se acumulan continuamente
- **Agnóstico de Token**: Funciona con cualquier token compatible con Soroban
- **Emisión de Eventos**: Eventos `staked`, `unstaked`, `rewards_claimed` y `rewards_added`
- **Gestión de TTL**: La TTL de almacenamiento de instancia se extiende en cada interacción

### Características del Contrato Subscription
- **Cargos Iniciados por Proveedor**: El proveedor de servicios extrae pagos en un intervalo de libro mayor configurable
- **Planes Controlados por Suscriptor**: Los suscriptores configuran su propio monto e intervalo; pueden cancelar en cualquier momento
- **Extracciones Basadas en Asignación**: Usa token `approve` + `transfer_from` — no se bloquean fondos por adelantado
- **Soporte para Resuscripción**: Los suscriptores cancelados pueden crear un nuevo plan sin reimplementar
- **Seguimiento de Estado**: Estado de suscripción (activo, libro mayor del último cargo) almacenado por suscriptor
- **Emisión de Eventos**: Eventos `subscribed`, `charged` y `cancelled` para seguimiento fuera de cadena
- **Gestión de TTL**: Tanto el almacenamiento de instancia como el persistente se extienden en cada interacción

### Características del Contrato Multisig
- **Autorización N-de-M**: Configura cualquier umbral válido entre firmantes únicos
- **Gestión de Firmantes**: Agregar o eliminar firmantes con cambios aprobados por umbral
- **Propuestas de Transacción**: Almacena contrato de destino, función y argumentos
- **Seguimiento de Firmas**: Prevenir firmas duplicadas y aprobaciones de no firmantes
- **Ejecución de Umbral**: Ejecutar llamadas propuestas solo después de suficientes firmas
- **Emisión de Eventos**: Inicialización, cambios de firmantes, firmas y ejecución emiten eventos

### Características del Contrato Timelock
- **Liberación Bloqueada en Tiempo**: Tokens retenidos hasta una secuencia de libro mayor especificada, luego liberados al beneficiario
- **Cancelación por Admin**: El admin puede cancelar y reclamar tokens en cualquier momento antes de la liberación
- **Liberación Abierta**: Una vez alcanzado el libro mayor de liberación, `release` es llamable por cualquiera
- **Agnóstico de Token**: Funciona con cualquier token compatible con Soroban
- **Emisión de Eventos**: Eventos `initialized`, `released` y `cancelled` para seguimiento fuera de cadena
- **Gestión de TTL**: La TTL de almacenamiento de instancia se extiende en cada interacción

### Características del Contrato NFT
- **Propiedad Única de Token**: Cada ID de token se asigna exactamente a un propietario rastreado en almacenamiento persistente
- **Minting Controlado por Admin**: Solo el admin puede acuñar nuevos tokens; el límite de suministro opcional se aplica en el momento de la acuñación
- **Operaciones Estándar**: `mint`, `transfer`, `burn`, `approve`, `transfer_from` que coinciden con la semántica ERC-721
- **Metadatos por Token**: Cada token tiene un URI asociado almacenado en cadena; la colección tiene nombre y símbolo
- **Sistema de Aprobación**: Las aprobaciones de un solo token se limpian automáticamente en transferencia o quema
- **Pruebas de Propiedad**: Suite de proptest verifica invariantes de suministro y corrección de propiedad
- **Emisión de Eventos**: Eventos `minted`, `transferred`, `burned` y `approved`

### Características del Contrato DAO
- **Votación Ponderada por Tokens**: El poder de voto es igual al saldo de tokens del votante en el momento del voto
- **Parámetros Configurables**: Período de votación (en libros mayor) y umbral de quórum establecidos en la inicialización
- **Ciclo de Vida de Propuesta**: `Activo → Ejecutado` (aprueba) o `Activo → Cancelado` (admin)
- **Quórum + Mayoría**: Las propuestas se ejecutan solo cuando los votos totales ≥ quórum Y sí > no
- **Prevención de Doble Voto**: Cada dirección puede votar exactamente una vez por propuesta
- **Emisión de Eventos**: Eventos `proposal_created`, `voted`, `prop_executed` y `prop_cancelled`
- **Gestión de TTL**: Los registros de propuesta y voto persistentes se incrementan en cada escritura

### Características del Contrato Swap
- **Intercambio Atómico**: Ambas transferencias de tokens ocurren en una sola transacción — sin rellenos parciales
- **Caducidad Basada en Plazo**: Los swaps caducan después de un libro mayor configurable; cualquiera puede cancelar para recuperar tokens de la Parte A
- **Control de Parte A**: La Parte A puede cancelar cualquier swap abierto antes de ser aceptado
- **Soporte Multi-Swap**: Múltiples swaps concurrentes rastreados por IDs de incremento automático
- **Agnóstico de Token**: Funciona con cualquier par de tokens compatibles con Soroban
- **Emisión de Eventos**: Eventos `swap_proposed`, `swap_accepted` y `swap_cancelled`

### Características del Contrato Oracle
- **Consumidor de Feed de Precios**: El admin empuja actualizaciones de precios; los consumidores leen vía `get_price`
- **Validación de Antigüedad**: `get_price` rechaza precios más antiguos que el umbral de libro mayor configurado
- **Actualizaciones Controladas por Admin**: Solo el admin puede empujar nuevos precios
- **Umbral Configurable**: El umbral de antigüedad se establece en la inicialización
- **Emisión de Eventos**: Eventos `initialized` y `price_updated`
- **Gestión de TTL**: La TTL de almacenamiento de instancia se extiende en cada interacción

### Características del Contrato Lottery
- **Aleatoriedad Commit-Reveal**: Admin se compromete a `hash(secreto ++ sal)` antes del sorteo, luego revela para probar la equidad
- **Compra de Boletos**: Cualquier dirección compra boletos antes de que el admin se comprometa
- **Selección Verificable del Ganador**: Índice del ganador derivado de SHA-256 de secreto revelado, sal y secuencia de libro mayor
- **Distribución del Fondo de Premios**: Fondo de boletos completo transferido al ganador atómicamente
- **Máquina de Estado**: Abierto → Comprometido → Dibujado — cada transición es irreversible
- **Emisión de Eventos**: Eventos `initialized`, `ticket_purchased`, `committed` y `winner_drawn`

Cada plantilla incluye:
- ✅ Implementación completa del contrato
- ✅ Pruebas unitarias exhaustivas (8+ casos de prueba cada una)
- ✅ Scripts de despliegue con ejemplos
- ✅ Ejemplos de uso y documentación

## 🛠 Requisitos Previos

- [Rust](https://rustup.rs/) **1.82.0** (fijado vía `rust-toolchain.toml` — `rustup` lo recoge automáticamente)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup#install-the-soroban-cli)
- [Docker](https://www.docker.com/) (para nodo Stellar local)

> **Opción sin instalación**: Abre este repositorio en un entorno preconfigurado con todas las herramientas listas — consulta la [Guía de Dev Container & Codespaces](docs/devcontainer.md).

## 🔄 Matriz de Compatibilidad

Este repositorio está fijado a `soroban-sdk = "=21.7.7"`. Cada versión principal del SDK está estrechamente acoplada a una versión de protocolo de red Stellar. Usa la tabla de abajo para elegir el SDK correcto para tu red de destino.

> ⚠️ **Siempre verifica la compatibilidad antes de desplegar en Mainnet.** Los contratos compilados contra SDK v21 no funcionarán en un nodo que ejecute Protocol 22 o posterior sin recompilación contra la versión SDK coincidente.

| versión soroban-sdk | Protocolo Stellar | Estado de Red | Notas |
|---------------------|-----------------|----------------|-------|
| `21.x` (este repositorio: `21.7.7`) | Protocolo 21 | Mainnet (Jun 2024) | secp256r1, extensión TTL separada para instancia/código |
| `22.x` | Protocolo 22 | Mainnet (Dic 2024) | Soporte de constructor, funciones de host BLS12-381 |
| `23.x` | Protocolo 23 "Whisk" | Mainnet (Sep 2025) | Eventos unificados (CAP-67), archivado de estado (CAP-62/66) |
| `25.x` | Protocolo 25 "X-Ray" | Mainnet (Ene 2026) | Operaciones de curva elíptica BN254, funciones hash Poseidon |
| `26.x` | Protocolo 26 "Yardstick" | Mainnet (May 2026) | Congelación de entradas de libro mayor, conversiones de direcciones muxed, ZK BN254 |

**Para actualizar este repositorio a un SDK más nuevo:**
1. Actualiza `soroban-sdk = "=<new-version>"` en `Cargo.toml`.
2. Actualiza `stellar-cli` a la versión coincidente (`cargo install stellar-cli --version <new-version>`).
3. Reconstruye todos los contratos y ejecuta la suite de pruebas completa.
4. Actualiza esta matriz y `docs/gas-costs.md` con la nueva versión de protocolo y cronograma de tarifas.

Para la tabla de versiones autoritaria, consulta [Versiones de Software de Stellar](https://developers.stellar.org/docs/networks/software-versions).

## 📖 Uso

### Construyendo Contratos

```bash
cd contracts/[template-name]
stellar contract build
```

### Ejecutando Pruebas

```bash
cd contracts/[template-name]
cargo test
```

### Despliegue en Testnet

```bash
cd contracts/[template-name]
./scripts/deploy.sh testnet
```

### Desarrollo Local

Inicia un nodo Stellar local con RPC Soroban:

```bash
docker compose up stellar-node
```

## ⚠️ Referencias de Errores

> Para detalles completos — causas, disparadores y pasos de resolución — consulta [docs/error-reference.md](docs/error-reference.md).

### Errores de Contrato Token (`TokenError`)

| Código | Nombre | Descripción |
|------|------|-------------|
| 1 | `InsufficientBalance` | El saldo del llamador es demasiado bajo para completar la transferencia o quema |
| 2 | `InsufficientAllowance` | La asignación aprobada es demasiado baja para la cantidad de `transfer_from` solicitada |
| 3 | `Unauthorized` | El llamador no es el admin o no tiene permiso para esta operación |
| 4 | `AlreadyInitialized` | Se llamó a `initialize` en un contrato que ya ha sido configurado |
| 5 | `NotInitialized` | Se intentó una operación antes de que el contrato fuera inicializado |
| 6 | `InvalidAmount` | La cantidad es cero, negativa o excede el suministro máximo configurado |
| 7 | `Overflow` | Ocurrió un desbordamiento aritmético durante un cálculo de saldo o suministro |

### Errores de Contrato Escrow (`EscrowError`)

| Código | Nombre | Descripción |
|------|------|-------------|
| 1 | `NotAuthorized` | El llamador no tiene permiso para invocar esta función (parte incorrecta o árbitro) |
| 2 | `InvalidState` | El escrow no está en el estado requerido para esta operación |
| 3 | `DeadlinePassed` | El plazo del escrow ya ha transcurrido; la operación ya no es válida |
| 4 | `DeadlineNotReached` | El plazo aún no ha llegado; intento prematuro de reembolso o reclamación de tiempo agotado |
| 5 | `AlreadyInitialized` | Se llamó a `initialize` en un escrow que ya está configurado |
| 6 | `NotInitialized` | Se intentó una operación antes de que el escrow fuera inicializado |
| 7 | `InsufficientFunds` | El saldo de token del comprador es demasiado bajo para cubrir la cantidad garantizada |
| 8 | `InvalidAmount` | La cantidad especificada es cero u otra forma inválida |
| 9 | `InvalidParties` | Las direcciones de comprador, vendedor o árbitro son inválidas o se contradicen |

### Errores de Contrato Vesting (`VestingError`)

| Código | Nombre | Descripción |
|------|------|-------------|
| 1 | `AlreadyInitialized` | Se llamó a `initialize` en un contrato que ya está configurado |
| 2 | `NotInitialized` | Se intentó una operación antes de que el contrato fuera inicializado |
| 3 | `Unauthorized` | El llamador no es el admin |
| 4 | `InvalidAmount` | La cantidad de vesting es cero o negativa |
| 5 | `InvalidSchedule` | `cliff_ledger` >= `end_ledger`, o `end_ledger` está en el pasado |
| 6 | `NothingToClaim` | Ningún token se ha adquirido desde la última reclamación (o la cantidad adquirida es cero) |
| 7 | `AlreadyRevoked` | Se llamó a `revoke` en un cronograma que ya ha sido revocado |

### Errores de Contrato Staking (`StakingError`)

| Código | Nombre | Descripción |
|------|------|-------------|
| 1 | `AlreadyInitialized` | Se llamó a `initialize` en un contrato que ya está configurado |
| 2 | `NotInitialized` | Se intentó una operación antes de que el contrato fuera inicializado |
| 3 | `Unauthorized` | El llamador no es el admin |
| 4 | `InvalidAmount` | La cantidad es cero o negativa |
| 5 | `NoStake` | El staker no tiene stake para desplegar o reclamar |
| 6 | `InsufficientStake` | La cantidad de despliegue solicitada excede el stake actual del staker |
| 7 | `NoRewards` | No hay recompensas disponibles para reclamar |

### Errores de Contrato Multisig (`MultisigError`)

| Código | Nombre | Descripción |
|------|------|-------------|
| 1 | `AlreadyInitialized` | Se llamó a `initialize` después de que el conjunto de firmantes ya estuviera configurado |
| 2 | `NotInitialized` | Se intentó una operación antes de que el multisig fuera inicializado |
| 3 | `InvalidThreshold` | El umbral es cero o mayor que el número de firmantes |
| 4 | `InvalidSigners` | Las listas de firmantes o aprobaciones están vacías o contienen duplicados |
| 5 | `NotSigner` | El llamador, aprobador o firmante no es parte del conjunto de firmantes de la cartera |
| 6 | `TransactionNotFound` | El ID de transacción solicitado no existe |
| 7 | `AlreadyExecuted` | La transacción ya ha sido ejecutada |
| 8 | `AlreadySigned` | El firmante ya aprobó la transacción |
| 9 | `ThresholdNotMet` | La transacción no tiene suficientes firmas para ejecutarse |
| 10 | `InsufficientApprovals` | El cambio de gestión de firmantes carece de suficientes aprobaciones de umbral |

## 📂 Ejemplos

Los ejemplos funcionales de extremo a extremo se proporcionan en el directorio `examples/`:

| Ejemplo | Descripción |
|---------|-------------|
| [`examples/typescript/index.js`](examples/typescript/index.js) | Script de Node.js — deplega token, acuña al comprador, ejecuta ciclo de vida completo de escrow |
| [`examples/shell/run.sh`](examples/shell/run.sh) | Script de shell equivalente usando Stellar CLI |

Ambos ejemplos apuntan a un nodo Stellar local. Inicia uno con `./scripts/local-net.sh start` antes de ejecutar.

### TypeScript

```bash
npm install @stellar/stellar-sdk
TOKEN_CONTRACT_ID=<id> ESCROW_CONTRACT_ID=<id> node examples/typescript/index.js
```

### Shell

```bash
./examples/shell/run.sh
```

---

## 🤝 Contribución

¡Bienvenidas las contribuciones! Consulta [CONTRIBUTING.md](CONTRIBUTING.md) para configuración de desarrollo, comandos de prueba, estilo de código y proceso de PR.

## 📚 Recursos

- [FAQ](docs/faq.md) — Preguntas frecuentes de desarrolladores: configuración, pruebas, despliegue, banderas de características, personalización de token
- [Arquitectura del Sistema](docs/architecture.md) — Diseño de alto nivel, relaciones de contratos, capas de almacenamiento, modelo de eventos y marco de administración
- [Mejores Prácticas de Seguridad](docs/security.md)
- [Guía de Integración](docs/integration-guide.md)
- [Guía de Despliegue](docs/deployment-guide.md)
- [Documentación de Soroban](https://soroban.stellar.org/docs)
- [Discord de Desarrolladores de Stellar](https://discord.gg/stellardev)
- [Ejemplos de Soroban](https://github.com/stellar/soroban-examples)
- [Cartera Freighter](https://freighter.app/)
- [Laboratorio de Stellar](https://laboratory.stellar.org/)
- [ADR de Cumplimiento de Interfaz de Token](docs/adr/0007-token-interface-compliance.md)
- [ADR de Diseño de Batch Mint](docs/adr/0009-batch-mint-design.md)
- [Registros de Decisiones Arquitectónicas](docs/adr/README.md)

## 📄 Licencia

Este proyecto está bajo la licencia Apache License 2.0 - consulta el archivo [LICENSE](LICENSE) para obtener más detalles.

---

**¿Listo para construir en Soroban?** ¡Comienza con cualquier plantilla y personalízala para tu caso de uso! 🚀
