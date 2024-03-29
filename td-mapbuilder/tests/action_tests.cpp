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

SCENARIO("Test delete mode", "[add]")
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


TEST_CASE("Test add mode", "[add]")
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


TEST_CASE("Test move mode", "[move]")
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

