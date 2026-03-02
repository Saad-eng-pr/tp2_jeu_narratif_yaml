// =============================================================================
// Module : error
// Rôle   : Définit tous les types d'erreur du projet.
//
// On sépare trois catégories d'erreurs :
//   - ParseError   : erreurs liées à l'entrée utilisateur (commande invalide).
//   - GameError     : erreurs métier en cours de partie (choix invalide, objet
//                     manquant, etc.).
//   - ValidationError : erreurs détectées lors de la validation du scénario
//                       YAML avant le début de la partie.
//
// Chaque type implémente `std::fmt::Display` pour fournir des messages lisibles.
// =============================================================================

use std::fmt;

/// Erreur de parsing d'une commande utilisateur.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// La ligne est vide.
    EmptyInput,
    /// La commande n'est pas reconnue.
    UnknownCommand(String),
    /// `choose` a été tapé sans numéro ou avec un argument non numérique.
    InvalidArgument(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Entrée vide. Tapez 'look' pour voir la scène."),
            ParseError::UnknownCommand(cmd) => write!(f, "Commande inconnue : '{}'", cmd),
            ParseError::InvalidArgument(msg) => write!(f, "Argument invalide : {}", msg),
        }
    }
}

/// Erreur métier survenant pendant l'exécution d'une commande en jeu.
#[derive(Debug, PartialEq)]
pub enum GameError {
    /// Le numéro de choix n'existe pas dans la scène courante.
    InvalidChoice(usize),
    /// Le joueur n'a pas l'objet requis pour emprunter ce chemin.
    MissingItem(String),
    /// La scène de destination n'existe pas (ne devrait pas arriver si la
    /// validation a été faite, mais on protège quand même).
    SceneNotFound(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::InvalidChoice(n) => write!(f, "Choix invalide : {}", n),
            GameError::MissingItem(item) => {
                write!(f, "Objet requis manquant : '{}'", item)
            }
            GameError::SceneNotFound(id) => write!(f, "Scène introuvable : '{}'", id),
        }
    }
}

/// Erreur de validation du scénario YAML (avant le lancement de la partie).
#[derive(Debug, PartialEq)]
pub enum ValidationError {
    /// La scène de départ n'existe pas dans la liste des scènes.
    StartSceneNotFound(String),
    /// Deux scènes portent le même identifiant.
    DuplicateSceneId(String),
    /// Un `choices.next` pointe vers une scène inexistante.
    BrokenLink { from: String, to: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::StartSceneNotFound(id) => {
                write!(f, "La scène de départ '{}' n'existe pas", id)
            }
            ValidationError::DuplicateSceneId(id) => {
                write!(f, "ID de scène dupliqué : '{}'", id)
            }
            ValidationError::BrokenLink { from, to } => {
                write!(f, "Lien cassé : la scène '{}' pointe vers '{}' qui n'existe pas", from, to)
            }
        }
    }
}
