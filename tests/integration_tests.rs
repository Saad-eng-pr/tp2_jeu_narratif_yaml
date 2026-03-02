// =============================================================================
// Tests d'intégration
//
// Couvrent les 5 scénarios de test minimum demandés :
//   1. Chemin nominal vers Victory.
//   2. Choix invalide : choose 99 → Err(InvalidChoice).
//   3. Choix conditionnel sans objet → Err(MissingItem).
//   4. Perte de HP (hp <= 0) → GameOver.
//   5. Scénario YAML invalide → erreur de validation.
// =============================================================================

use tp2_jeu_narratif::command::{parse_command, CommandOutcome};
use tp2_jeu_narratif::error::{GameError, ParseError};
use tp2_jeu_narratif::scenario::Scenario;
use tp2_jeu_narratif::state::GameState;

// -----------------------------------------------------------------------------
// Scénario YAML de test (embarqué pour ne pas dépendre d'un fichier externe)
// -----------------------------------------------------------------------------

const TEST_YAML: &str = r#"
start_scene: entrance
initial_hp: 10

scenes:
  - id: entrance
    title: Porte Principale
    text: La pluie frappe les vitres.
    choices:
      - label: Entrer dans le hall
        next: hall
      - label: Renoncer
        next: street

  - id: hall
    title: Hall Central
    text: Un ecran clignote.
    choices:
      - label: Fouiller le stockage
        next: storage
      - label: Forcer l'acces au toit
        next: roof
        required_item: badge

  - id: storage
    title: Stockage
    text: Des caisses renversees.
    found_item: badge
    choices:
      - label: Retourner au hall
        next: hall

  - id: roof
    title: Toit
    text: Vous activez la balise.
    ending: victory

  - id: street
    title: Rue
    text: Vous fuyez.
    ending: escape

  - id: danger
    title: Zone Dangereuse
    text: Tres dangereux.
    hp_delta: -20
    choices:
      - label: Continuer
        next: entrance
"#;

// =============================================================================
// Test 1 : Chemin nominal vers Victory
//
// Parcours : entrance → hall → storage (ramasse badge) → hall → roof (victory)
// =============================================================================
#[test]
fn test_chemin_nominal_victory() {
    let scenario = Scenario::load_from_str(TEST_YAML).unwrap();
    let mut state = GameState::new(&scenario);

    // entrance → hall (choix 1)
    let cmd = parse_command("choose 1").unwrap();
    let result = cmd.execute(&scenario, &mut state).unwrap();
    assert_eq!(result, CommandOutcome::SceneChanged);
    assert_eq!(state.current_scene, "hall");

    // hall → storage (choix 1)
    let cmd = parse_command("choose 1").unwrap();
    let result = cmd.execute(&scenario, &mut state).unwrap();
    assert_eq!(result, CommandOutcome::SceneChanged);
    assert_eq!(state.current_scene, "storage");

    // On doit avoir ramassé le badge
    assert!(state.has_item("badge"));

    // storage → hall (choix 1)
    let cmd = parse_command("choose 1").unwrap();
    let result = cmd.execute(&scenario, &mut state).unwrap();
    assert_eq!(result, CommandOutcome::SceneChanged);
    assert_eq!(state.current_scene, "hall");

    // hall → roof (choix 2, nécessite badge — on l'a)
    let cmd = parse_command("choose 2").unwrap();
    let result = cmd.execute(&scenario, &mut state).unwrap();
    assert_eq!(result, CommandOutcome::GameEnded("victory".to_string()));
    assert_eq!(state.current_scene, "roof");
}

// =============================================================================
// Test 2 : Choix invalide → Err(InvalidChoice)
// =============================================================================
#[test]
fn test_choix_invalide() {
    let scenario = Scenario::load_from_str(TEST_YAML).unwrap();
    let mut state = GameState::new(&scenario);

    let cmd = parse_command("choose 99").unwrap();
    let result = cmd.execute(&scenario, &mut state);
    assert_eq!(result, Err(GameError::InvalidChoice(99)));
}

// =============================================================================
// Test 3 : Choix conditionnel sans objet → Err(MissingItem)
// =============================================================================
#[test]
fn test_missing_item() {
    let scenario = Scenario::load_from_str(TEST_YAML).unwrap();
    let mut state = GameState::new(&scenario);

    // Aller au hall
    let cmd = parse_command("choose 1").unwrap();
    cmd.execute(&scenario, &mut state).unwrap();
    assert_eq!(state.current_scene, "hall");

    // Essayer d'aller au toit sans badge → MissingItem
    let cmd = parse_command("choose 2").unwrap();
    let result = cmd.execute(&scenario, &mut state);
    assert_eq!(result, Err(GameError::MissingItem("badge".to_string())));
}

// =============================================================================
// Test 4 : Perte de HP → GameOver
// =============================================================================
#[test]
fn test_game_over_hp() {
    let scenario = Scenario::load_from_str(TEST_YAML).unwrap();
    let mut state = GameState::new(&scenario);

    // Aller directement à la zone dangereuse (on triche en changeant la scène)
    state.current_scene = "danger".to_string();

    // danger → entrance (choix 1), mais hp_delta de "danger" ne s'applique pas
    // car on y est déjà. On simule en mettant les HP bas puis en "choisissant" 
    // d'aller vers entrance depuis danger. En fait, le hp_delta s'applique à 
    // la scène d'arrivée. Testons autrement :

    // Mettre les HP à 5 et naviguer vers "danger" depuis une scène voisine
    // On va créer un scénario spécifique pour ce test.
    let hp_yaml = r#"
start_scene: safe
initial_hp: 5

scenes:
  - id: safe
    title: Safe
    text: Safe zone.
    choices:
      - label: Go to danger
        next: lethal

  - id: lethal
    title: Lethal
    text: You die here.
    hp_delta: -10
    choices:
      - label: Back
        next: safe
"#;
    let scenario2 = Scenario::load_from_str(hp_yaml).unwrap();
    let mut state2 = GameState::new(&scenario2);

    // safe → lethal (hp_delta = -10, HP = 5 → -5 ≤ 0 → game_over)
    let cmd = parse_command("choose 1").unwrap();
    let result = cmd.execute(&scenario2, &mut state2).unwrap();
    assert_eq!(result, CommandOutcome::GameEnded("game_over".to_string()));
    assert!(state2.hp <= 0);
}

// =============================================================================
// Test 5 : Scénario YAML invalide → erreurs de validation
// =============================================================================
#[test]
fn test_validation_start_scene_missing() {
    let yaml = r#"
start_scene: nonexistent
initial_hp: 10
scenes:
  - id: a
    title: A
    text: Scene A.
"#;
    let scenario = Scenario::load_from_str(yaml).unwrap();
    let errors = scenario.validate();
    assert!(!errors.is_empty());
    // Vérifier qu'on a bien une erreur StartSceneNotFound
    assert!(errors.iter().any(|e| matches!(e,
        tp2_jeu_narratif::error::ValidationError::StartSceneNotFound(_)
    )));
}

#[test]
fn test_validation_duplicate_id() {
    let yaml = r#"
start_scene: a
initial_hp: 10
scenes:
  - id: a
    title: A
    text: Scene A.
  - id: a
    title: A bis
    text: Scene A duplicate.
"#;
    let scenario = Scenario::load_from_str(yaml).unwrap();
    let errors = scenario.validate();
    assert!(errors.iter().any(|e| matches!(e,
        tp2_jeu_narratif::error::ValidationError::DuplicateSceneId(_)
    )));
}

#[test]
fn test_validation_broken_link() {
    let yaml = r#"
start_scene: a
initial_hp: 10
scenes:
  - id: a
    title: A
    text: Scene A.
    choices:
      - label: Go nowhere
        next: does_not_exist
"#;
    let scenario = Scenario::load_from_str(yaml).unwrap();
    let errors = scenario.validate();
    assert!(errors.iter().any(|e| matches!(e,
        tp2_jeu_narratif::error::ValidationError::BrokenLink { .. }
    )));
}

// =============================================================================
// Tests du parser de commandes
// =============================================================================
#[test]
fn test_parse_look() {
    assert!(parse_command("look").is_ok());
}

#[test]
fn test_parse_choose_valid() {
    assert!(parse_command("choose 1").is_ok());
}

#[test]
fn test_parse_choose_missing_arg() {
    let result = parse_command("choose");
    assert!(matches!(result, Err(ParseError::InvalidArgument(_))));
}

#[test]
fn test_parse_choose_non_numeric() {
    let result = parse_command("choose abc");
    assert!(matches!(result, Err(ParseError::InvalidArgument(_))));
}

#[test]
fn test_parse_unknown_command() {
    let result = parse_command("fly");
    assert!(matches!(result, Err(ParseError::UnknownCommand(_))));
}

#[test]
fn test_parse_empty_input() {
    let result = parse_command("");
    assert!(matches!(result, Err(ParseError::EmptyInput)));
}

#[test]
fn test_inventory_command() {
    let scenario = Scenario::load_from_str(TEST_YAML).unwrap();
    let mut state = GameState::new(&scenario);

    // Inventaire vide
    let cmd = parse_command("inventory").unwrap();
    let result = cmd.execute(&scenario, &mut state).unwrap();
    assert!(matches!(result, CommandOutcome::Message(_)));

    // Ajouter un objet et vérifier
    state.add_item("badge".to_string());
    let cmd = parse_command("inventory").unwrap();
    let result = cmd.execute(&scenario, &mut state).unwrap();
    if let CommandOutcome::Message(msg) = result {
        assert!(msg.contains("badge"));
    } else {
        panic!("Expected Message outcome");
    }
}

#[test]
fn test_status_command() {
    let scenario = Scenario::load_from_str(TEST_YAML).unwrap();
    let mut state = GameState::new(&scenario);

    let cmd = parse_command("status").unwrap();
    let result = cmd.execute(&scenario, &mut state).unwrap();
    if let CommandOutcome::Message(msg) = result {
        assert!(msg.contains("10")); // HP initial
        assert!(msg.contains("entrance"));
    } else {
        panic!("Expected Message outcome");
    }
}

#[test]
fn test_quit_command() {
    let scenario = Scenario::load_from_str(TEST_YAML).unwrap();
    let mut state = GameState::new(&scenario);

    let cmd = parse_command("quit").unwrap();
    let result = cmd.execute(&scenario, &mut state).unwrap();
    assert_eq!(result, CommandOutcome::Quit);
    assert!(state.quit);
}

// =============================================================================
// Test avec le fichier story.yaml réel
// =============================================================================
#[test]
fn test_load_real_story_yaml() {
    let scenario = Scenario::load_from_file("story.yaml").unwrap();
    let errors = scenario.validate();
    assert!(errors.is_empty(), "Le story.yaml fourni doit être valide : {:?}", errors);
    assert_eq!(scenario.start_scene, "entrance");
    assert_eq!(scenario.initial_hp, 10);
    assert_eq!(scenario.scenes.len(), 9);
}
