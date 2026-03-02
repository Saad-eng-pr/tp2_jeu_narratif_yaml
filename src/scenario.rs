// =============================================================================
// Module : scenario
// Rôle   : Modélise le scénario YAML et fournit le chargement + la validation.
//
// Architecture :
//   - `Choice`   : un choix affiché au joueur (label, destination, objet requis).
//   - `Scene`    : une scène du jeu (texte, choix, objet trouvé, HP delta, fin).
//   - `Scenario` : le conteneur racine (scène de départ, HP initiaux, scènes).
//
// Le parsing est délégué à `serde_yaml` grâce aux derives `Deserialize`.
// La validation vérifie la cohérence du graphe de scènes avant de jouer.
// =============================================================================

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;

use crate::error::ValidationError;

// -----------------------------------------------------------------------------
// Structures de données
// -----------------------------------------------------------------------------

/// Un choix proposé au joueur dans une scène.
#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    /// Texte affiché pour ce choix (ex: "Entrer dans le hall").
    pub label: String,
    /// Identifiant de la scène de destination.
    pub next: String,
    /// Objet optionnel requis pour pouvoir choisir cette option.
    pub required_item: Option<String>,
}

/// Une scène du jeu narratif.
#[derive(Debug, Deserialize, Clone)]
pub struct Scene {
    /// Identifiant unique de la scène (ex: "hall").
    pub id: String,
    /// Titre affiché en haut de la scène.
    pub title: String,
    /// Texte narratif décrivant la scène.
    pub text: String,
    /// Liste des choix disponibles (vide si la scène est une fin).
    #[serde(default)]
    pub choices: Vec<Choice>,
    /// Objet trouvé en arrivant dans cette scène (ajouté à l'inventaire).
    pub found_item: Option<String>,
    /// Modification des HP en arrivant dans cette scène (ex: -2).
    pub hp_delta: Option<i32>,
    /// Si présent, la scène est une fin de partie ("victory", "escape", "defeat").
    pub ending: Option<String>,
}

/// Le scénario complet chargé depuis le fichier YAML.
#[derive(Debug, Deserialize, Clone)]
pub struct Scenario {
    /// Identifiant de la scène de départ.
    pub start_scene: String,
    /// Points de vie initiaux du joueur.
    pub initial_hp: i32,
    /// Liste de toutes les scènes du jeu.
    pub scenes: Vec<Scene>,
}

// -----------------------------------------------------------------------------
// Chargement depuis un fichier
// -----------------------------------------------------------------------------

impl Scenario {
    /// Charge un scénario depuis un fichier YAML.
    ///
    /// Retourne une erreur `String` si le fichier est introuvable ou mal formé.
    pub fn load_from_file(path: &str) -> Result<Scenario, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Impossible de lire '{}' : {}", path, e))?;
        Self::load_from_str(&content)
    }

    /// Charge un scénario depuis une chaîne YAML (utile pour les tests).
    pub fn load_from_str(yaml: &str) -> Result<Scenario, String> {
        serde_yaml::from_str(yaml)
            .map_err(|e| format!("Erreur de parsing YAML : {}", e))
    }

    /// Recherche une scène par son identifiant.
    pub fn find_scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.iter().find(|s| s.id == id)
    }

    // -------------------------------------------------------------------------
    // Validation
    // -------------------------------------------------------------------------

    /// Valide la cohérence du scénario.
    ///
    /// Vérifie :
    ///   1. La scène de départ existe.
    ///   2. Toutes les scènes ont un identifiant unique.
    ///   3. Tous les `choices.next` pointent vers des scènes existantes.
    ///
    /// Retourne la liste de toutes les erreurs trouvées (vide si tout est OK).
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Construire l'ensemble des IDs existants
        let mut seen_ids = HashSet::new();
        for scene in &self.scenes {
            if !seen_ids.insert(&scene.id) {
                errors.push(ValidationError::DuplicateSceneId(scene.id.clone()));
            }
        }

        // Vérifier que la scène de départ existe
        if !seen_ids.contains(&self.start_scene) {
            errors.push(ValidationError::StartSceneNotFound(self.start_scene.clone()));
        }

        // Vérifier que chaque destination de choix existe
        for scene in &self.scenes {
            for choice in &scene.choices {
                if !seen_ids.contains(&choice.next) {
                    errors.push(ValidationError::BrokenLink {
                        from: scene.id.clone(),
                        to: choice.next.clone(),
                    });
                }
            }
        }

        errors
    }
}
