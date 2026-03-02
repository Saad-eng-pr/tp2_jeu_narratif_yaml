// =============================================================================
// main.rs — Point d'entrée du binaire
//
// Ce fichier ne contient PAS de logique métier (conformément aux consignes).
// Il se contente de :
//   1. Charger et valider le scénario YAML.
//   2. Afficher la scène initiale.
//   3. Boucler sur l'entrée utilisateur : parser → exécuter → afficher.
//   4. Gérer les fins de partie (victoire, défaite, fuite, quit).
// =============================================================================

use std::io::{self, Write};

use tp2_jeu_narratif::command::{parse_command, CommandOutcome};
use tp2_jeu_narratif::scenario::Scenario;
use tp2_jeu_narratif::state::GameState;

/// Chemin par défaut du fichier scénario.
const STORY_FILE: &str = "story.yaml";

fn main() {
    // -------------------------------------------------------------------------
    // 1. Chargement du scénario
    // -------------------------------------------------------------------------
    let scenario = match Scenario::load_from_file(STORY_FILE) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur : {}", e);
            std::process::exit(1);
        }
    };

    // -------------------------------------------------------------------------
    // 2. Validation du scénario
    // -------------------------------------------------------------------------
    let errors = scenario.validate();
    if !errors.is_empty() {
        eprintln!("Scénario invalide :");
        for err in &errors {
            eprintln!("  - {}", err);
        }
        std::process::exit(1);
    }

    // -------------------------------------------------------------------------
    // 3. Initialisation de l'état de jeu
    // -------------------------------------------------------------------------
    let mut state = GameState::new(&scenario);

    println!("=== Jeu Narratif ===");
    println!("Commandes : look, choose <n>, inventory, status, quit\n");

    // Afficher la scène de départ
    display_scene(&scenario, &state);

    // -------------------------------------------------------------------------
    // 4. Boucle de jeu
    // -------------------------------------------------------------------------
    loop {
        // Lire l'entrée utilisateur
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        // Parser la commande
        let command = match parse_command(&input) {
            Ok(cmd) => cmd,
            Err(e) => {
                println!("{}\n", e);
                continue;
            }
        };

        // Exécuter la commande
        match command.execute(&scenario, &mut state) {
            Ok(outcome) => match outcome {
                CommandOutcome::DisplayScene => {
                    display_scene(&scenario, &state);
                }
                CommandOutcome::SceneChanged => {
                    println!();
                    display_scene(&scenario, &state);
                }
                CommandOutcome::Message(msg) => {
                    println!("{}\n", msg);
                }
                CommandOutcome::GameEnded(ending) => {
                    println!();
                    display_scene(&scenario, &state);
                    match ending.as_str() {
                        "victory" => println!("*** VICTOIRE ! Félicitations ! ***\n"),
                        "escape" => println!("*** Vous vous êtes échappé. Fin. ***\n"),
                        "defeat" | "game_over" => println!("*** GAME OVER ***\n"),
                        other => println!("*** Fin : {} ***\n", other),
                    }
                    break;
                }
                CommandOutcome::Quit => {
                    println!("Au revoir !\n");
                    break;
                }
            },
            Err(e) => {
                println!("Erreur : {}\n", e);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Affichage d'une scène
// -----------------------------------------------------------------------------

/// Affiche le titre, le texte et les choix de la scène courante.
fn display_scene(scenario: &Scenario, state: &GameState) {
    if let Some(scene) = scenario.find_scene(&state.current_scene) {
        println!("--- {} ---", scene.title);
        println!("{}", scene.text);

        // Afficher l'objet trouvé
        if let Some(ref item) = scene.found_item {
            println!("[Vous trouvez : {}]", item);
        }

        // Afficher le delta de HP
        if let Some(delta) = scene.hp_delta {
            if delta < 0 {
                println!("[Vous perdez {} HP — HP restants : {}]", -delta, state.hp);
            } else if delta > 0 {
                println!("[Vous gagnez {} HP — HP : {}]", delta, state.hp);
            }
        }

        // Afficher les choix
        if !scene.choices.is_empty() {
            println!("\nChoix :");
            for (i, choice) in scene.choices.iter().enumerate() {
                let lock = if choice.required_item.is_some() {
                    " [nécessite un objet]"
                } else {
                    ""
                };
                println!("  {}. {}{}", i + 1, choice.label, lock);
            }
        }
        println!();
    }
}
