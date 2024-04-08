#include <catch2/catch_test_macros.hpp>
#include <variant>
#include "../src/action.hpp"
#include "../src/game_entity.hpp"
#include "SFML/System/Vector2.hpp"
#include "../src/cursor_and_keys.hpp"


gameEntity setupGameEntity(std::vector<sf::Vector2f> positions)
{
    gameEntity game_entity{25, sf::Color(0,0,139, 128), sf::Color(0,0, 200)};
    for(sf::Vector2f& pos: positions)
    {
        game_entity.addEntity(pos);
    }
    return game_entity;
}

void selectAll(gameEntity& game_entity)
{
    for(auto& entity: game_entity.entities)
    {
        entity.is_selected = true;
    }
}

int countSelected(const gameEntity& game_entity)
{
    int num_selected = 0;
    for(auto& entity: game_entity.entities)
    {
        num_selected += entity.is_selected;
    }
    return num_selected;
}

std::string toString(const Mode& mode)
{
    if(std::holds_alternative<del>(mode)){return "delete";}
    else if(std::holds_alternative<move>(mode)){return "move";}
    else if(std::holds_alternative<add>(mode)){return "add";}
    else if(std::holds_alternative<select_click>(mode)){return "select_click";}
    else if(std::holds_alternative<select_drag>(mode)){return "select_drag";}
    if(std::holds_alternative<none>(mode)){return "none";}
    else return "unknown";
}

SCENARIO("Test delete mode", "[action]")
{
    std::vector<sf::Vector2f> positions {{0, 0}, {100, 100},{200, 100}, {500, 500}};

    MouseEvent mouse_event;
    ActionOptions action_opts;
    ActionMode action;
    KeyboardEvent keyboard_event;
    std::vector<Mode> test_modes = {select_click{}, select_drag{}, add{}, move{}, del{}, none{}};

    for(auto& test_mode: test_modes)
    {
        GIVEN("Mode is " + toString(test_mode)){
            gameEntity game_entity = setupGameEntity(positions);
            REQUIRE(game_entity.entities.size() == 4);

            game_entity.deselectAll();

            action_opts.mode = test_mode;

            keyboard_event.del_down = true;
            mouse_event.pressed = true;

            // Should not delete if left click is pressed
                
            WHEN("Delete is pressed, but mouse key is pressed")
            {
                action.compute(game_entity, keyboard_event, mouse_event, action_opts);
                THEN("Mode does not change"){
                    CHECK(toString(test_mode) == toString(action_opts.mode));
                }
            }
            
            mouse_event.pressed = false;
            game_entity.entities[1].is_selected = true;
            game_entity.entities[2].is_selected = true;
            action.compute(game_entity, keyboard_event, mouse_event, action_opts);
            WHEN("Delete is pressed, and mouse key is not pressed"){
                THEN("Mode is delete, and selected entities are deleted")
                {
                    CHECK(std::holds_alternative<del>(action_opts.mode));

                    executeAction(game_entity, keyboard_event, mouse_event, action_opts);
                    CHECK(game_entity.entities.size() == 2);
                    CHECK(countSelected(game_entity) == 0);
                }
            }

            // Recheck to test if this deletes any other entities
            WHEN("Delete is pressed with no selected entities"){
                THEN("No further entities get deleted"){
                    executeAction(game_entity, keyboard_event, mouse_event, action_opts);
                    CHECK(game_entity.entities.size() == 2);
                    CHECK(countSelected(game_entity) == 0);
                }
            }
        }
    }
}


TEST_CASE("Test add mode", "[action]")
{
    std::vector<sf::Vector2f> positions {{0, 0}, {100, 100},{200, 100}, {500, 500}};
    gameEntity game_entity = setupGameEntity(positions);
    REQUIRE(game_entity.entities.size() == 4);

    MouseEvent mouse_event;
    ActionOptions action_opts;
    ActionMode action;
    KeyboardEvent keyboard_event;
    std::vector<Mode> test_modes = {select_click{}, select_drag{}, add{}, move{}, del{}, none{}};

    for(auto& test_mode: test_modes)
    {
        game_entity.deselectAll();
        action_opts.mode = test_mode;
        mouse_event.position = {1000, 1000};
        mouse_event.moved_while_pressed = false;
        mouse_event.released_this_frame = true;

        action.compute(game_entity, keyboard_event, mouse_event, action_opts);
        CHECK(std::holds_alternative<add>(action_opts.mode));
        
        executeAction(game_entity, keyboard_event, mouse_event, action_opts);
        CHECK(game_entity.entities.size() == 5);

        action_opts.mode = test_mode;
        mouse_event.position = {0, 0};
        mouse_event.pressed_this_frame = true;
        keyboard_event.ctrl_down = true;
        selectAll(game_entity);

        action.compute(game_entity, keyboard_event, mouse_event, action_opts);
        CHECK(std::holds_alternative<add>(action_opts.mode));
        
        executeAction(game_entity, keyboard_event, mouse_event, action_opts);
        CHECK(game_entity.entities.size() == 6);
        CHECK(countSelected(game_entity) == 1);
        game_entity.deleteEntity(5);
        game_entity.deleteEntity(4);
    }
}

std::vector<Mode> testActionSequence(ActionMode& action, gameEntity& game_entity, KeyboardEvent& keyboard_event, MouseEvent& mouse_event, ActionOptions& action_opts, const sf::Vector2f& new_position)
{
    // Imitates a simple 'click'-'hold'-'release' sequence, with or without movement. Can this be generalized? Ideally, I would use the update-methods of MouseEvent and KeyboardEvent

    sf::Vector2f orig_position = mouse_event.position;

    std::vector<Mode> mode_history;

    mouse_event.pressed_this_frame = true;
    mouse_event.pressed = true;
    mouse_event.click_pos = mouse_event.position;

    action.compute(game_entity, keyboard_event, mouse_event, action_opts);
    mode_history.emplace_back(action_opts.mode);

    executeAction(game_entity, keyboard_event, mouse_event, action_opts);


    mouse_event.pressed_this_frame = false;
    mouse_event.moved_while_pressed = orig_position.x != new_position.x || orig_position.y != new_position.y ;

    action.compute(game_entity, keyboard_event, mouse_event, action_opts);
    mode_history.emplace_back(action_opts.mode);

    executeAction(game_entity, keyboard_event, mouse_event, action_opts);

    mouse_event.position = new_position;
    mouse_event.cursor_movement = new_position - orig_position;
    action.compute(game_entity, keyboard_event, mouse_event, action_opts);
    mode_history.emplace_back(action_opts.mode);

    executeAction(game_entity, keyboard_event, mouse_event, action_opts);

    mouse_event.pressed = false;
    mouse_event.released_this_frame = true;

    action.compute(game_entity, keyboard_event, mouse_event, action_opts);
    mode_history.emplace_back(action_opts.mode);

    executeAction(game_entity, keyboard_event, mouse_event, action_opts);
    mouse_event.position = orig_position;

    mouse_event.released_this_frame = false;
    mouse_event.moved_while_pressed = false;

    return mode_history;
}


SCENARIO("Test move mode", "[action]")
{
    std::vector<sf::Vector2f> positions {{0, 0}, {100, 100},{200, 100}, {500, 500}};

    MouseEvent mouse_event;
    ActionOptions action_opts;
    ActionMode action;
    KeyboardEvent keyboard_event;
    std::vector<Mode> test_modes = {select_click{}, select_drag{}, add{}, move{}, del{}, none{}};

    for(auto& test_mode: test_modes)
    {
        GIVEN("Test mode is " + toString(test_mode)){        
            gameEntity game_entity = setupGameEntity(positions);
            game_entity.deselectAll();
            REQUIRE(game_entity.entities.size() == 4);

            action_opts.mode = test_mode;
            
            size_t test_index = 1;
            sf::Vector2f mouse_increment {100, 200};
            mouse_event.position = positions[test_index];
       
            WHEN("Unselected entity is pressed, and immediately moved"){
                std::vector<Mode> intermediate_mode = testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position + mouse_increment);            
                THEN("Mode is move and test_index is moved accordingly")
                {
                    CHECK_FALSE(std::holds_alternative<move>(intermediate_mode[0]));
                    CHECK(std::holds_alternative<move>(action_opts.mode));
                    for(size_t i = 0; i < game_entity.entities.size(); i++)
                    {
                        CHECK(game_entity.entities[i].position == positions[i] + mouse_increment * float(i == test_index));
                        CHECK(game_entity.entities[i].is_selected == (i == test_index));

                        //Reset position for next test
                        game_entity.entities[i].position = positions[i];
                    }
                }
            }

            selectAll(game_entity);
            test_index = 2;
            mouse_increment = sf::Vector2f(75, 215);
            mouse_event.position = game_entity.entities[test_index].position;
            WHEN("All entities are selected, pressed and moved"){
                testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position + mouse_increment);            
                THEN("Mode is move and all are moved accordingly")
                {
                    CHECK(game_entity.entities.size() == 4);
                    CHECK(std::holds_alternative<move>(action_opts.mode));
                    for(size_t i = 0; i < game_entity.entities.size(); i++)
                    {
                        CHECK(game_entity.entities[i].position == positions[i] + mouse_increment);
                        CHECK(game_entity.entities[i].is_selected);

                        //Reset position for next test
                        game_entity.entities[i].position = positions[i];
                    }
                }
            }
        }
    }
}


SCENARIO("Test select_click mode", "[action]")
{
    std::vector<sf::Vector2f> positions {{0, 0}, {100, 100},{200, 100}, {500, 500}};

    MouseEvent mouse_event;
    ActionOptions action_opts;
    ActionMode action;
    KeyboardEvent keyboard_event;
    std::vector<Mode> test_modes = {select_click{}, select_drag{}, add{}, move{}, del{}, none{}};

    for(auto& test_mode: test_modes)
    {
        GIVEN("Test mode is " + toString(test_mode)){        
            action_opts.mode = test_mode;
            
            gameEntity game_entity = setupGameEntity(positions);
            REQUIRE(game_entity.entities.size() == 4);

            // Check esc to deselect all
            CHECK(countSelected(game_entity) > 0);
            keyboard_event.esc_down = true;
            action.compute(game_entity, keyboard_event, mouse_event, action_opts);
            CHECK(std::holds_alternative<select_click>(action_opts.mode));
            executeAction(game_entity, keyboard_event, mouse_event, action_opts);
            CHECK(countSelected(game_entity) == 0);
            CHECK(game_entity.entities.size() == 4);

            action_opts.mode = test_mode;
            keyboard_event.esc_down = false;
            
            size_t test_index = 2;
            mouse_event.position = positions[test_index];
       
            WHEN("Unselected entity is pressed and no entities are selected"){
                std::vector<Mode> intermediate_modes = testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position);
                THEN("Mode is select_click, nothing is moved and only the test index is selected")
                {
                    for(const Mode& mode: intermediate_modes)
                    {
                        CHECK(std::holds_alternative<select_click>(mode));
                    }
                    CHECK(std::holds_alternative<select_click>(action_opts.mode));
                    for(size_t i = 0; i < game_entity.entities.size(); i++)
                    {
                        CHECK(game_entity.entities[i].position == positions[i]);
                        CHECK(game_entity.entities[i].is_selected == (i == test_index));
                    }
                }
            }

            WHEN("Unselected entity is pressed and another entity is selected"){
                test_index = 3;
                action_opts.mode = test_mode;
                mouse_event.position = positions[test_index];
                testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position);
                THEN("Mode is select_click and only one entity is selected")
                {
                    CHECK(std::holds_alternative<select_click>(action_opts.mode));
                    for(size_t i = 0; i < game_entity.entities.size(); i++)
                    {
                        CHECK(game_entity.entities[i].position == positions[i]);
                        CHECK(game_entity.entities[i].is_selected == (i == test_index));
                    }
                }
            }

            WHEN("Unselected entity is pressed and another entity is selected and shift is pressed"){
                test_index = 1;
                action_opts.mode = test_mode;
                keyboard_event.shift_down = true;
                mouse_event.position = positions[test_index];
                game_entity.entities[3].is_selected = true;
                testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position);
                THEN("Mode is select_click and entity 1 and 3 are selected")
                {
                    CHECK(std::holds_alternative<select_click>(action_opts.mode));
                    for(size_t i = 0; i < game_entity.entities.size(); i++)
                    {
                        CHECK(game_entity.entities[i].position == positions[i]);
                        CHECK(game_entity.entities[i].is_selected == (i == 1 || i == 3));
                    }
                }
            }

            WHEN("Selected entity is pressed"){
                game_entity.deselectAll();
                test_index = 3;
                action_opts.mode = test_mode;
                keyboard_event.shift_down = false;
                mouse_event.position = positions[test_index];
                game_entity.entities[3].is_selected = true;

                testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position);
                THEN("Mode is select_click and no entities are selected")
                {
                    CHECK(std::holds_alternative<select_click>(action_opts.mode));
                    CHECK(countSelected(game_entity) == 0);
                }
            }

            WHEN("All entities are selected, and entity is pressed but shift is down"){
                selectAll(game_entity);
                test_index = 0;
                action_opts.mode = test_mode;
                keyboard_event.shift_down = true;
                mouse_event.position = positions[test_index];

                testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position);
                THEN("Mode is select_click and only pressed entity is unselected")
                {
                    CHECK(std::holds_alternative<select_click>(action_opts.mode));
                    for(size_t i = 0; i < game_entity.entities.size(); i++)
                    {
                        CHECK(game_entity.entities[i].is_selected == (i != test_index));
                    }
                }
            }

            WHEN("All entities are selected, and no-intersection press but shift is down"){
                selectAll(game_entity);
                action_opts.mode = test_mode;
                keyboard_event.shift_down = true;
                mouse_event.position = sf::Vector2f(1000, 924);

                testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position);
                THEN("Mode is select_click and only pressed entity is unselected")
                {
                    CHECK(std::holds_alternative<select_click>(action_opts.mode));
                    CHECK(countSelected(game_entity) == game_entity.entities.size());
                }
            }

            WHEN("All entities are selected, and no-intersection press"){
                selectAll(game_entity);
                action_opts.mode = test_mode;
                keyboard_event.shift_down = false;
                mouse_event.position = sf::Vector2f(-156, 2498);

                testActionSequence(action, game_entity, keyboard_event, mouse_event, action_opts, mouse_event.position);
                THEN("Mode is select_click and only pressed entity is unselected")
                {
                    CHECK(std::holds_alternative<select_click>(action_opts.mode));
                    CHECK(countSelected(game_entity) == 0);
                }
            }
        }
    }
}
