// =============================================================================
// Module : command
// Rôle   : Implémente le Command Pattern pour traiter les entrées utilisateur.
//
// Principe du Command Pattern :
//   Chaque commande tapée par l'utilisateur est transformée en un objet
//   (struct) qui implémente le trait `GameCommand`. Cela permet de :
//     - séparer le parsing de l'exécution,
//     - tester chaque commande indépendamment,
//     - ajouter de nouvelles commandes sans modifier le code existant.
//
// Trait `GameCommand` :
//   - `execute(scenario, state)` -> `Result<CommandOutcome, GameError>`
//
// Structs de commande :
//   - LookCommand      : réaffiche la scène courante
//   - ChooseCommand(n) : choisit l'option numéro n
//   - InventoryCommand : affiche l'inventaire
//   - StatusCommand    : affiche HP et scène courante
//   - QuitCommand      : quitte la partie
//
// Fonction `parse_command` :
//   Transforme une ligne de texte en `Box<dyn GameCommand>`.
// =============================================================================

use crate::error::{GameError, ParseError};
use crate::scenario::Scenario;
use crate::state::GameState;

// -----------------------------------------------------------------------------
// Résultat d'une commande
// -----------------------------------------------------------------------------

/// Résultat retourné après l'exécution réussie d'une commande.
/// Le moteur de jeu (main loop) utilise ce résultat pour décider quoi afficher.
#[derive(Debug, PartialEq)]
pub enum CommandOutcome {
    /// Afficher la scène courante (texte + choix).
    DisplayScene,
    /// Afficher un message libre (inventaire, statut, etc.).
    Message(String),
    /// Le joueur a changé de scène (afficher la nouvelle scène).
    SceneChanged,
    /// La partie est terminée (victoire, défaite, fuite…).
    GameEnded(String),
    /// Le joueur veut quitter.
    Quit,
}

// -----------------------------------------------------------------------------
// Trait GameCommand
// -----------------------------------------------------------------------------

/// Trait commun à toutes les commandes du jeu.
///
/// Chaque commande reçoit une référence au scénario (lecture seule)
/// et une référence mutable à l'état de jeu.
pub trait GameCommand {
    fn execute(
        &self,
        scenario: &Scenario,
        state: &mut GameState,
    ) -> Result<CommandOutcome, GameError>;
}

// -----------------------------------------------------------------------------
// LookCommand — réaffiche la scène courante
// -----------------------------------------------------------------------------

/// Commande `look` : réaffiche la scène où se trouve le joueur.
pub struct LookCommand;

impl GameCommand for LookCommand {
    fn execute(
        &self,
        scenario: &Scenario,
        state: &mut GameState,
    ) -> Result<CommandOutcome, GameError> {
        // Vérifier que la scène existe (sécurité)
        scenario
            .find_scene(&state.current_scene)
            .ok_or_else(|| GameError::SceneNotFound(state.current_scene.clone()))?;
        Ok(CommandOutcome::DisplayScene)
    }
}

// -----------------------------------------------------------------------------
// ChooseCommand — choisit une option
// -----------------------------------------------------------------------------

/// Commande `choose <n>` : le joueur choisit l'option numéro `n` (1-indexé).
pub struct ChooseCommand {
    pub choice_index: usize, // 1-indexé (comme affiché au joueur)
}

impl GameCommand for ChooseCommand {
    fn execute(
        &self,
        scenario: &Scenario,
        state: &mut GameState,
    ) -> Result<CommandOutcome, GameError> {
        // Trouver la scène courante
        let scene = scenario
            .find_scene(&state.current_scene)
            .ok_or_else(|| GameError::SceneNotFound(state.current_scene.clone()))?;

        // Vérifier que le numéro de choix est valide (1-indexé)
        if self.choice_index == 0 || self.choice_index > scene.choices.len() {
            return Err(GameError::InvalidChoice(self.choice_index));
        }

        let choice = &scene.choices[self.choice_index - 1];

        // Vérifier l'objet requis
        if let Some(ref required) = choice.required_item {
            if !state.has_item(required) {
                return Err(GameError::MissingItem(required.clone()));
            }
        }

        // Changer de scène
        state.current_scene = choice.next.clone();

        // Trouver la nouvelle scène
        let new_scene = scenario
            .find_scene(&state.current_scene)
            .ok_or_else(|| GameError::SceneNotFound(state.current_scene.clone()))?;

        // Appliquer le delta de HP
        if let Some(delta) = new_scene.hp_delta {
            state.hp += delta;
        }

        // Ramasser l'objet trouvé
        if let Some(ref item) = new_scene.found_item {
            state.add_item(item.clone());
        }

        // Vérifier si le joueur est mort (HP <= 0)
        if state.hp <= 0 {
            return Ok(CommandOutcome::GameEnded("game_over".to_string()));
        }

        // Vérifier si c'est une scène de fin
        if let Some(ref ending) = new_scene.ending {
            return Ok(CommandOutcome::GameEnded(ending.clone()));
        }

        Ok(CommandOutcome::SceneChanged)
    }
}

// -----------------------------------------------------------------------------
// InventoryCommand — affiche l'inventaire
// -----------------------------------------------------------------------------

/// Commande `inventory` : liste les objets du joueur.
pub struct InventoryCommand;

impl GameCommand for InventoryCommand {
    fn execute(
        &self,
        _scenario: &Scenario,
        state: &mut GameState,
    ) -> Result<CommandOutcome, GameError> {
        if state.inventory.is_empty() {
            Ok(CommandOutcome::Message(
                "Inventaire vide.".to_string(),
            ))
        } else {
            let items = state
                .inventory
                .iter()
                .enumerate()
                .map(|(i, item)| format!("  {}. {}", i + 1, item))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(CommandOutcome::Message(format!("Inventaire :\n{}", items)))
        }
    }
}

// -----------------------------------------------------------------------------
// StatusCommand — affiche HP + scène
// -----------------------------------------------------------------------------

/// Commande `status` : affiche les HP et la scène courante.
pub struct StatusCommand;

impl GameCommand for StatusCommand {
    fn execute(
        &self,
        scenario: &Scenario,
        state: &mut GameState,
    ) -> Result<CommandOutcome, GameError> {
        let scene = scenario
            .find_scene(&state.current_scene)
            .ok_or_else(|| GameError::SceneNotFound(state.current_scene.clone()))?;
        let msg = format!(
            "HP : {}\nScène : {} ({})",
            state.hp, scene.title, scene.id
        );
        Ok(CommandOutcome::Message(msg))
    }
}

// -----------------------------------------------------------------------------
// QuitCommand — quitte la partie
// -----------------------------------------------------------------------------

/// Commande `quit` : le joueur quitte la partie.
pub struct QuitCommand;

impl GameCommand for QuitCommand {
    fn execute(
        &self,
        _scenario: &Scenario,
        state: &mut GameState,
    ) -> Result<CommandOutcome, GameError> {
        state.quit = true;
        Ok(CommandOutcome::Quit)
    }
}

// -----------------------------------------------------------------------------
// parse_command — parsing de l'entrée utilisateur
// -----------------------------------------------------------------------------

/// Transforme une ligne de texte saisie par l'utilisateur en objet commande.
///
/// Commandes reconnues :
///   - `look`
///   - `choose <n>` (n = numéro du choix, commence à 1)
///   - `inventory`
///   - `status`
///   - `quit`
///
/// Retourne `Err(ParseError)` si la commande est invalide.
pub fn parse_command(line: &str) -> Result<Box<dyn GameCommand>, ParseError> {
    let line = line.trim();

    if line.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Séparer la commande de ses arguments
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next().unwrap().to_lowercase();
    let arg = parts.next().map(|s| s.trim());

    match cmd.as_str() {
        "look" => Ok(Box::new(LookCommand)),
        "choose" => {
            let arg = arg.ok_or_else(|| {
                ParseError::InvalidArgument("usage: choose <numéro>".to_string())
            })?;
            let n: usize = arg.parse().map_err(|_| {
                ParseError::InvalidArgument(format!("'{}' n'est pas un numéro valide", arg))
            })?;
            Ok(Box::new(ChooseCommand { choice_index: n }))
        }
        "inventory" => Ok(Box::new(InventoryCommand)),
        "status" => Ok(Box::new(StatusCommand)),
        "quit" => Ok(Box::new(QuitCommand)),
        other => Err(ParseError::UnknownCommand(other.to_string())),
    }
}
