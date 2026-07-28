# Modèles de Contrats Soroban

[![CI](https://github.com/Fidelis900/soroban-starter-kit/actions/workflows/ci.yml/badge.svg)](https://github.com/Fidelis900/soroban-starter-kit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Fidelis900/soroban-starter-kit/branch/main/graph/badge.svg)](https://codecov.io/gh/Fidelis900/soroban-starter-kit)
[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/Fidelis900/soroban-starter-kit)

Une collection organisée de modèles de contrats intelligents Soroban prêts pour la production. Ces modèles aident les développeurs à démarrer rapidement des cas d'usage courants sur Soroban (la plateforme de contrats intelligents de Stellar) pour la DeFi, les paiements, la gouvernance, et plus encore.

> **English version**: [README.md](README.md)
> **También disponible en**: [Español 🇪🇸](README.es.md)

## 🚀 Démarrage Rapide

```bash
# Cloner le dépôt
git clone https://github.com/your-username/soroban-contract-templates.git
cd soroban-contract-templates

# Construire tous les contrats
make build

# Exécuter les tests
make test

# Relancer les tests automatiquement à chaque modification (nécessite cargo-watch)
make watch

# Déployer sur testnet
make deploy-testnet

# Voir toutes les commandes disponibles
make help
```

Ou utilisez `just` (voir [dev-environment.md](docs/dev-environment.md) pour l'installation) :

```bash
just build
just test
just deploy-testnet
just --list
```

## 📦 Modèles de Contrats

| Modèle | Description | Cas d'Usage | Statut |
|----------|-------------|-----------|---------|
| **Token** | Token fongible personnalisé avec contrôles mint/burn/admin | Tokens DeFi, tokens de gouvernance, tokens utilitaires | ✅ Complet |
| **Escrow** | Séquestre entre deux parties avec délai et mécanisme de remboursement | Échanges P2P, paiements de services, paiements par étapes | ✅ Complet |
| **Vesting** | Acquisition de tokens avec cliff + calendrier de libération linéaire | Allocations d'équipe, blocages d'investisseurs, attributions employés | ✅ Complet |
| **Staking** | Staking de tokens avec distribution proportionnelle des récompenses | Rendement DeFi, incitations de protocole, minage de liquidité | ✅ Complet |
| **Multisig** | Portefeuille N-parmi-M pour appels de contrats approuvés par seuil | Trésoreries de DAO, portefeuilles d'équipe, administration partagée | ✅ Complet |
| **Subscription** | Contrat de paiement récurrent par prélèvement de tokens | Facturation SaaS, paiements en continu, frais d'adhésion | ✅ Complet |
| **Timelock** | Libération de tokens verrouillée dans le temps vers un bénéficiaire | Blocages de tokens d'équipe, paiements différés, timelocks de gouvernance | ✅ Complet |
| **NFT** | Token non fongible avec mint administré et plafond d'offre optionnel | Objets de collection numériques, propriété on-chain, tokens d'accès | ✅ Complet |
| **DAO** | Gouvernance on-chain avec vote pondéré par les tokens | Mises à jour de protocole, gestion de trésorerie, décisions communautaires | ✅ Complet |
| **Swap** | Échange atomique de tokens entre deux parties avec délai | Échange de tokens P2P, transactions OTC, swaps DeFi sans confiance | ✅ Complet |
| **Oracle** | Consommateur de flux de prix avec validation de fraîcheur | Flux de prix DeFi, consommation de données on-chain, vérifications de fraîcheur | ✅ Complet |
| **Lottery** | Loterie on-chain vérifiable avec aléatoire commit-reveal | Tombolas, tirages équitables, distribution décentralisée de prix | ✅ Complet |

### Fonctionnalités du Contrat Token
- **Interface Standard** : Compatibilité complète avec le token Soroban
- **Contrôles Administratifs** : Gestion du mint, burn et de l'admin
- **Support des Métadonnées** : Nom, symbole et décimales
- **Système d'Allocation** : Fonctionnalité approve et transfer_from
- **Émission d'Événements** : Toutes les opérations émettent des événements pour le suivi
- **Gestion des Erreurs** : Types d'erreurs personnalisés pour un meilleur débogage

### Fonctionnalités du Contrat Escrow
- **Sécurité à Deux Parties** : Transactions sécurisées acheteur-vendeur
- **Protection par Délai** : Remboursements automatiques après le délai
- **Support d'Arbitre** : Résolution de litiges par un tiers
- **Gestion d'État** : Cycle de vie clair de la transaction
- **Agnostique au Token** : Fonctionne avec n'importe quel token Soroban
- **Émission d'Événements** : Toutes les opérations émettent des événements pour le suivi

### Fonctionnalités du Contrat Vesting
- **Calendrier Cliff + Linéaire** : Les tokens se libèrent linéairement entre `cliff_ledger` et `end_ledger`
- **Révocation par l'Admin** : L'admin peut annuler les tokens non acquis à tout moment ; les tokens acquis restent réclamables
- **Réclamations Incrémentales** : Le bénéficiaire réclame les tokens accumulés à la demande
- **Agnostique au Token** : Fonctionne avec n'importe quel token compatible Soroban
- **Émission d'Événements** : Événements `initialized`, `claimed` et `revoked` pour le suivi hors chaîne
- **Gestion du TTL** : Le TTL du stockage d'instance est prolongé à chaque interaction

### Fonctionnalités du Contrat Staking
- **Récompenses Proportionnelles** : Les récompenses sont distribuées au prorata de la part de chaque staker dans le pool
- **Accumulateur de Récompense par Token** : Modèle d'accumulateur global économe en gas ; aucune boucle par staker
- **Tokens de Stake / Récompense Séparés** : Le token de stake et le token de récompense peuvent être identiques ou différents
- **Dépôts de Récompense par l'Admin** : L'admin appelle `add_rewards` pour alimenter le pool de récompenses à tout moment
- **Réclamations Incrémentales** : Les stakers appellent `claim_rewards` indépendamment ; les récompenses s'accumulent en continu
- **Agnostique au Token** : Fonctionne avec n'importe quel token compatible Soroban
- **Émission d'Événements** : Événements `staked`, `unstaked`, `rewards_claimed` et `rewards_added`
- **Gestion du TTL** : Le TTL du stockage d'instance est prolongé à chaque interaction

### Fonctionnalités du Contrat Subscription
- **Prélèvements Initiés par le Fournisseur** : Le fournisseur de service prélève les paiements à un intervalle de ledger configurable
- **Plans Contrôlés par l'Abonné** : Les abonnés définissent leur propre montant et intervalle ; annulation possible à tout moment
- **Prélèvements par Allocation** : Utilise `approve` + `transfer_from` du token — aucun fonds n'est bloqué à l'avance
- **Support de Réabonnement** : Les abonnés annulés peuvent créer un nouveau plan sans redéployer
- **Suivi d'État** : L'état de l'abonnement (actif, dernier ledger facturé) est stocké par abonné
- **Émission d'Événements** : Événements `subscribed`, `charged` et `cancelled` pour le suivi hors chaîne
- **Gestion du TTL** : Le stockage d'instance et persistant sont tous deux prolongés à chaque interaction

### Fonctionnalités du Contrat Multisig
- **Autorisation N-parmi-M** : Configurez n'importe quel seuil valide parmi des signataires uniques
- **Gestion des Signataires** : Ajout ou suppression de signataires via des changements approuvés par seuil
- **Propositions de Transaction** : Stocke le contrat cible, la fonction et les arguments
- **Suivi des Signatures** : Empêche les signatures dupliquées et les approbations de non-signataires
- **Exécution par Seuil** : Exécute les appels proposés uniquement après avoir réuni suffisamment de signatures
- **Émission d'Événements** : L'initialisation, les changements de signataires, les signatures et l'exécution émettent des événements

### Fonctionnalités du Contrat Timelock
- **Libération Verrouillée dans le Temps** : Les tokens sont retenus jusqu'à un numéro de séquence de ledger spécifié, puis libérés au bénéficiaire
- **Annulation par l'Admin** : L'admin peut annuler et récupérer les tokens à tout moment avant la libération
- **Libération Ouverte** : Une fois le ledger de libération atteint, `release` est appelable par n'importe qui
- **Agnostique au Token** : Fonctionne avec n'importe quel token compatible Soroban
- **Émission d'Événements** : Événements `initialized`, `released` et `cancelled` pour le suivi hors chaîne
- **Gestion du TTL** : Le TTL du stockage d'instance est prolongé à chaque interaction

### Fonctionnalités du Contrat NFT
- **Propriété Unique du Token** : Chaque ID de token correspond exactement à un propriétaire suivi en stockage persistant
- **Mint Contrôlé par l'Admin** : Seul l'admin peut créer de nouveaux tokens ; un plafond d'offre optionnel est appliqué au moment du mint
- **Opérations Standard** : `mint`, `transfer`, `burn`, `approve`, `transfer_from` correspondant à la sémantique ERC-721
- **Métadonnées par Token** : Chaque token possède une URI associée stockée on-chain ; la collection a un nom et un symbole
- **Système d'Approbation** : Les approbations d'un seul token sont automatiquement effacées lors d'un transfert ou d'un burn
- **Tests de Propriétés** : La suite proptest vérifie les invariants d'offre et l'exactitude de la propriété
- **Émission d'Événements** : Événements `minted`, `transferred`, `burned` et `approved`

### Fonctionnalités du Contrat DAO
- **Vote Pondéré par les Tokens** : Le pouvoir de vote est égal au solde de tokens du votant au moment du vote
- **Paramètres Configurables** : Période de vote (en ledgers) et seuil de quorum définis à l'initialisation
- **Cycle de Vie de la Proposition** : `Active → Exécutée` (adoptée) ou `Active → Annulée` (admin)
- **Quorum + Majorité** : Les propositions ne s'exécutent que lorsque le total des votes ≥ quorum ET oui > non
- **Prévention du Double Vote** : Chaque adresse ne peut voter qu'une seule fois par proposition
- **Émission d'Événements** : Événements `proposal_created`, `voted`, `prop_executed` et `prop_cancelled`
- **Gestion du TTL** : Les enregistrements persistants de proposition et de vote sont prolongés à chaque écriture

### Fonctionnalités du Contrat Swap
- **Échange Atomique** : Les deux transferts de tokens se produisent en une seule transaction — aucun remplissage partiel
- **Expiration par Délai** : Les swaps expirent après un ledger configurable ; n'importe qui peut annuler pour récupérer les tokens de la partie A
- **Contrôle de la Partie A** : La partie A peut annuler tout swap ouvert avant qu'il ne soit accepté
- **Support Multi-Swap** : Plusieurs swaps concurrents suivis par des IDs auto-incrémentés
- **Agnostique au Token** : Fonctionne avec n'importe quelle paire de tokens compatibles Soroban
- **Émission d'Événements** : Événements `swap_proposed`, `swap_accepted` et `swap_cancelled`

### Fonctionnalités du Contrat Oracle
- **Consommateur de Flux de Prix** : L'admin pousse les mises à jour de prix ; les consommateurs lisent via `get_price`
- **Validation de Fraîcheur** : `get_price` rejette les prix plus anciens que le seuil de ledger configuré
- **Mises à Jour Contrôlées par l'Admin** : Seul l'admin peut pousser de nouveaux prix
- **Seuil Configurable** : Le seuil de fraîcheur est défini à l'initialisation
- **Émission d'Événements** : Événements `initialized` et `price_updated`
- **Gestion du TTL** : Le TTL du stockage d'instance est prolongé à chaque interaction

### Fonctionnalités du Contrat Lottery
- **Aléatoire Commit-Reveal** : L'admin s'engage sur `hash(secret ++ salt)` avant le tirage, puis révèle pour prouver l'équité
- **Achat de Billets** : N'importe quelle adresse achète des billets avant que l'admin ne s'engage
- **Sélection Vérifiable du Gagnant** : L'index du gagnant est dérivé du SHA-256 du secret révélé, du sel et de la séquence de ledger
- **Distribution du Pool de Prix** : Le pool complet des billets est transféré au gagnant de manière atomique
- **Machine à États** : Ouvert → Engagé → Tiré — chaque transition est irréversible
- **Émission d'Événements** : Événements `initialized`, `ticket_purchased`, `committed` et `winner_drawn`

Chaque modèle inclut :
- ✅ Implémentation complète du contrat
- ✅ Tests unitaires exhaustifs (8+ cas de test chacun)
- ✅ Scripts de déploiement avec exemples
- ✅ Exemples d'utilisation et documentation

## 🛠 Prérequis

- [Rust](https://rustup.rs/) **1.82.0** (fixé via `rust-toolchain.toml` — `rustup` le détecte automatiquement)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup#install-the-soroban-cli)
- [Docker](https://www.docker.com/) (pour un nœud Stellar local)

> **Option sans installation :** Ouvrez ce dépôt dans un environnement préconfiguré avec tous les outils prêts — voir le [Guide Dev Container & Codespaces](docs/devcontainer.md).

### Configuration VS Code

Un fichier [.vscode/extensions.json](.vscode/extensions.json) est inclus avec des extensions recommandées pour une expérience de développement cohérente. VS Code vous proposera de les installer à l'ouverture du projet, ou vous pouvez les installer manuellement :

| Extension | Utilité |
|-----------|---------|
| `rust-lang.rust-analyzer` | Serveur de langage Rust (complétion, aller à la définition, erreurs en ligne) |
| `tamasfe.even-better-toml` | Coloration syntaxique et validation pour `Cargo.toml` |
| `serayuzgur.crates` | Indications de version de crate en ligne et avertissements de dépendances obsolètes |
| `usernamehw.errorlens` | Messages de diagnostic affichés directement dans l'éditeur |

## 🔄 Matrice de Compatibilité

Ce dépôt est fixé sur `soroban-sdk = "=21.7.7"`. Chaque version majeure du SDK est étroitement couplée à une version du protocole réseau Stellar. Utilisez le tableau ci-dessous pour choisir le bon SDK pour votre réseau cible.

> ⚠️ **Vérifiez toujours la compatibilité avant de déployer sur Mainnet.** Les contrats compilés avec le SDK v21 ne fonctionneront pas sur un nœud exécutant le Protocol 22 ou une version ultérieure sans recompilation avec la version de SDK correspondante.

| Version soroban-sdk | Protocole Stellar | Statut Réseau | Notes |
|---------------------|-----------------|----------------|-------|
| `21.x` (ce dépôt : `21.7.7`) | Protocol 21 | Mainnet (juin 2024) | secp256r1, extension TTL séparée pour l'instance/le code |
| `22.x` | Protocol 22 | Mainnet (déc. 2024) | Support des constructeurs, fonctions hôtes BLS12-381 |
| `23.x` | Protocol 23 « Whisk » | Mainnet (sept. 2025) | Événements unifiés (CAP-67), archivage d'état (CAP-62/66) |
| `25.x` | Protocol 25 « X-Ray » | Mainnet (jan. 2026) | Opérations de courbe elliptique BN254, fonctions de hachage Poseidon |
| `26.x` | Protocol 26 « Yardstick » | Mainnet (mai 2026) | Gel des entrées de ledger, conversions d'adresses muxed, ZK BN254 |

**Pour mettre à jour ce dépôt vers un SDK plus récent :**
1. Mettez à jour `soroban-sdk = "=<nouvelle-version>"` dans `Cargo.toml`.
2. Mettez à jour `stellar-cli` vers la version correspondante (`cargo install stellar-cli --version <nouvelle-version>`).
3. Reconstruisez tous les contrats et exécutez la suite de tests complète.
4. Mettez à jour cette matrice et `docs/gas-costs.md` avec la nouvelle version de protocole et le nouveau barème de frais.

Pour la table de versions faisant autorité, voir [Stellar Software Versions](https://developers.stellar.org/docs/networks/software-versions).

## 📖 Utilisation

### Construire les Contrats

```bash
cd contracts/[nom-du-modele]
stellar contract build
```

### Exécuter les Tests

```bash
cd contracts/[nom-du-modele]
cargo test
```

### Déployer sur Testnet

```bash
cd contracts/[nom-du-modele]
./scripts/deploy.sh testnet
```

### Développement Local

Démarrez un nœud Stellar local avec RPC Soroban :

```bash
docker compose up stellar-node
```

### Créer un Nouveau Contrat

Générez un nouveau contrat à partir du squelette `contracts/common` avec une seule commande :

```bash
./scripts/new-contract.sh <nom-du-contrat>
```

**Exemple** — créer un contrat `price-feed` :

```bash
./scripts/new-contract.sh price-feed
```

Cela crée `contracts/price-feed/` (avec `Cargo.toml` et `src/lib.rs`) et l'enregistre dans le workspace. Vérifiez immédiatement qu'il compile et que son test passe :

```bash
cargo check -p soroban-price-feed
cargo test  -p soroban-price-feed
```

Remplacez ensuite la méthode `hello()` du squelette dans `contracts/price-feed/src/lib.rs` par la logique de votre contrat.

## ⚠️ Référence des Erreurs

> Pour tous les détails — causes, déclencheurs et étapes de résolution — voir [docs/error-reference.md](docs/error-reference.md).

### Erreurs du Contrat Token (`TokenError`)

| Code | Nom | Description |
|------|------|-------------|
| 1 | `InsufficientBalance` | Le solde de l'appelant est trop faible pour effectuer le transfert ou le burn |
| 2 | `InsufficientAllowance` | L'allocation approuvée est trop faible pour le montant `transfer_from` demandé |
| 3 | `Unauthorized` | L'appelant n'est pas l'admin ou n'a pas la permission pour cette opération |
| 4 | `AlreadyInitialized` | `initialize` a été appelé sur un contrat déjà configuré |
| 5 | `NotInitialized` | Une opération a été tentée avant l'initialisation du contrat |
| 6 | `InvalidAmount` | Le montant est nul, négatif, ou dépasse l'offre maximale configurée |
| 7 | `Overflow` | Un débordement arithmétique s'est produit lors d'un calcul de solde ou d'offre |

### Erreurs du Contrat Escrow (`EscrowError`)

| Code | Nom | Description |
|------|------|-------------|
| 1 | `NotAuthorized` | L'appelant n'est pas autorisé à invoquer cette fonction (mauvaise partie ou arbitre) |
| 2 | `InvalidState` | Le séquestre n'est pas dans l'état requis pour cette opération |
| 3 | `DeadlinePassed` | Le délai du séquestre est déjà écoulé ; l'opération n'est plus valide |
| 4 | `DeadlineNotReached` | Le délai n'est pas encore passé ; tentative prématurée de remboursement ou de réclamation de timeout |
| 5 | `AlreadyInitialized` | `initialize` a été appelé sur un séquestre déjà configuré |
| 6 | `NotInitialized` | Une opération a été tentée avant l'initialisation du séquestre |
| 7 | `InsufficientFunds` | Le solde de tokens de l'acheteur est trop faible pour couvrir le montant séquestré |
| 8 | `InvalidAmount` | Le montant spécifié est nul ou autrement invalide |
| 9 | `InvalidParties` | Les adresses de l'acheteur, du vendeur ou de l'arbitre sont invalides ou en conflit |

### Erreurs du Contrat Vesting (`VestingError`)

| Code | Nom | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | `initialize` a été appelé sur un contrat déjà configuré |
| 2 | `NotInitialized` | Une opération a été tentée avant l'initialisation du contrat |
| 3 | `Unauthorized` | L'appelant n'est pas l'admin |
| 4 | `InvalidAmount` | Le montant d'acquisition est nul ou négatif |
| 5 | `InvalidSchedule` | `cliff_ledger` >= `end_ledger`, ou `end_ledger` est dans le passé |
| 6 | `NothingToClaim` | Aucun token n'a été acquis depuis la dernière réclamation (ou le montant acquis est nul) |
| 7 | `AlreadyRevoked` | `revoke` a été appelé sur un calendrier déjà révoqué |

### Erreurs du Contrat Staking (`StakingError`)

| Code | Nom | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | `initialize` a été appelé sur un contrat déjà configuré |
| 2 | `NotInitialized` | Une opération a été tentée avant l'initialisation du contrat |
| 3 | `Unauthorized` | L'appelant n'est pas l'admin |
| 4 | `InvalidAmount` | Le montant est nul ou négatif |
| 5 | `NoStake` | Le staker n'a aucun stake à retirer ou dont réclamer des récompenses |
| 6 | `InsufficientStake` | Le montant de retrait demandé dépasse le stake actuel du staker |
| 7 | `NoRewards` | Aucune récompense disponible à réclamer |

### Erreurs du Contrat Multisig (`MultisigError`)

| Code | Nom | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | `initialize` a été appelé après que l'ensemble des signataires ait déjà été configuré |
| 2 | `NotInitialized` | Une opération a été tentée avant l'initialisation du multisig |
| 3 | `InvalidThreshold` | Le seuil est nul ou supérieur au nombre de signataires |
| 4 | `InvalidSigners` | Les listes de signataires ou d'approbations sont vides ou contiennent des doublons |
| 5 | `NotSigner` | L'appelant, l'approbateur ou le signataire ne fait pas partie de l'ensemble des signataires du portefeuille |
| 6 | `TransactionNotFound` | L'ID de transaction demandé n'existe pas |
| 7 | `AlreadyExecuted` | La transaction a déjà été exécutée |
| 8 | `AlreadySigned` | Le signataire a déjà approuvé la transaction |
| 9 | `ThresholdNotMet` | La transaction n'a pas suffisamment de signatures pour s'exécuter |
| 10 | `InsufficientApprovals` | Le changement de gestion des signataires manque d'approbations de seuil suffisantes |

## 📂 Exemples

Des exemples fonctionnels de bout en bout sont fournis dans le répertoire `examples/` :

| Exemple | Description |
|---------|-------------|
| [`examples/typescript/index.js`](examples/typescript/index.js) | Script Node.js — déploie un token, en crédite l'acheteur, exécute le cycle de vie complet d'un séquestre |
| [`examples/shell/run.sh`](examples/shell/run.sh) | Script shell équivalent utilisant le Stellar CLI |

Les deux exemples ciblent un nœud Stellar local. Démarrez-en un avec `./scripts/local-net.sh start` avant de les exécuter.

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

## 🤝 Contribuer

Les contributions sont les bienvenues ! Voir [CONTRIBUTING.md](CONTRIBUTING.md) pour la configuration de développement, les commandes de test, le style de code et le processus de PR.

Un grand merci à tous nos [contributeurs](CONTRIBUTORS.md) qui ont aidé à améliorer ce projet !

## 📚 Ressources

- [Arborescence du Répertoire](docs/directory-tree.md) — Rôle de chaque répertoire du dépôt
- [FAQ](docs/faq.md) — Questions fréquentes des développeurs : configuration, tests, déploiement, feature flags, personnalisation des tokens
- [Architecture du Système](docs/architecture.md) — Conception de haut niveau, relations entre contrats, niveaux de stockage, modèle d'événements et cadre d'administration
- [Référence de l'API des Contrats](docs/contract-api.md) — API publique complète de tous les contrats (paramètres, types de retour, erreurs)
- [Guide de Mise à Niveau](docs/upgrade-guide.md) — Mise à niveau WASM on-chain étape par étape avec timelock et rotation de clés
- [Bonnes Pratiques de Sécurité](docs/security.md)
- [Guide d'Intégration](docs/integration-guide.md)
- [Guide de Déploiement](docs/deployment-guide.md)
- [Documentation Soroban](https://soroban.stellar.org/docs)
- [Discord des Développeurs Stellar](https://discord.gg/stellardev)
- [Exemples Soroban](https://github.com/stellar/soroban-examples)
- [Portefeuille Freighter](https://freighter.app/)
- [Stellar Laboratory](https://laboratory.stellar.org/)
- [ADR de Conformité de l'Interface Token](docs/adr/0007-token-interface-compliance.md)
- [ADR de Conception du Batch Mint](docs/adr/0009-batch-mint-design.md)
- [Registres de Décisions d'Architecture](docs/adr/README.md)

## 📄 Licence

Ce projet est sous licence Apache License 2.0 - voir le fichier [LICENSE](LICENSE) pour plus de détails.

---

**Prêt à construire sur Soroban ?** Commencez avec n'importe quel modèle et personnalisez-le pour votre cas d'usage ! 🚀
