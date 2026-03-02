// =============================================================================
// lib.rs — Point d'entrée de la bibliothèque
//
// Ce fichier déclare les modules du projet et les rend accessibles :
//   - depuis `main.rs` (le binaire)
//   - depuis les tests d'intégration (dossier `tests/`)
//
// Organisation des modules :
//   error    : types d'erreur (ParseError, GameError, ValidationError)
//   scenario : structures YAML + chargement + validation
//   state    : état mutable de la partie (HP, inventaire, scène courante)
//   command  : Command Pattern (trait + structs + parse_command)
// =============================================================================

pub mod error;
pub mod scenario;
pub mod state;
pub mod command;
