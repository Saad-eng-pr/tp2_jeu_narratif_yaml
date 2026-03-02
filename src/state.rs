// =============================================================================
// Module : state
// Rôle   : Gère l'état mutable de la partie en cours.
//
// `GameState` contient tout ce qui change pendant la partie :
//   - la scène courante
//   - les HP du joueur
//   - l'inventaire (liste d'objets trouvés)
//   - un drapeau indiquant si le joueur veut quitter
//
// Ce module est volontairement simple : il ne contient aucune logique métier.
// Les commandes modifient cet état via le module `command`.
// =============================================================================

use crate::scenario::Scenario;

/// État courant d'une partie en cours.
#[derive(Debug, Clone)]
pub struct GameState {
    /// Identifiant de la scène où se trouve le joueur.
    pub current_scene: String,
    /// Points de vie restants.
    pub hp: i32,
    /// Objets collectés par le joueur.
    pub inventory: Vec<String>,
    /// Indique si le joueur a demandé à quitter.
    pub quit: bool,
}

impl GameState {
    /// Crée un nouvel état de jeu à partir d'un scénario validé.
    pub fn new(scenario: &Scenario) -> Self {
        GameState {
            current_scene: scenario.start_scene.clone(),
            hp: scenario.initial_hp,
            inventory: Vec::new(),
            quit: false,
        }
    }

    /// Vérifie si le joueur possède un objet donné.
    pub fn has_item(&self, item: &str) -> bool {
        self.inventory.iter().any(|i| i == item)
    }

    /// Ajoute un objet à l'inventaire (s'il n'est pas déjà présent).
    pub fn add_item(&mut self, item: String) {
        if !self.has_item(&item) {
            self.inventory.push(item);
        }
    }
}
