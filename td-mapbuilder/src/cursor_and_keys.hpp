#pragma once
#include "SFML/Window/Keyboard.hpp"
#include "SFML/Window/Mouse.hpp"
#include "SFML/System/Vector2.hpp"
#include <vector>
#include "globals.hpp"
#include "vector.hpp"

class MouseEvent
{          
    public:
    sf::Mouse mouse;
    bool pressed = false;
    bool released_this_frame = false;
    bool pressed_this_frame = false;
    bool moved_while_pressed = false;
    sf::Vector2f position = {0,0};
    sf::Vector2f click_pos = {0,0};
    sf::Vector2f cursor_movement = {0,0};
    void update()
    {
        sf::Vector2f new_pos = vectorRescaler(sf::Vector2f(mouse.getPosition(globals.window)),sf::Vector2f(globals.window.getSize()), globals.map_texture_size);
        cursor_movement = new_pos - position;
        position = new_pos;
        released_this_frame = false;
        pressed_this_frame = false;
        moved_while_pressed = pressed && (cursor_movement != sf::Vector2f(0,0) || moved_while_pressed);
        if (mouse.isButtonPressed(mouse.Left))
        {
            if(!pressed)
            {
                pressed_this_frame = true;
                click_pos = position;    
            }
            pressed = true;
        }
        else
        {
            if(pressed){released_this_frame = true;}
            pressed = false;
        }
    }
};

class oneFrameKey{
    private:
    bool pressed = false;
    std::vector<sf::Keyboard::Key> keys; 
    public:
    bool this_frame = false;
    oneFrameKey(std::vector<sf::Keyboard::Key> keys) : keys(std::move(keys)){};
    void Update(sf::Keyboard& keyboard)
    {
        this_frame = false;
        for(sf::Keyboard::Key& key: keys)
        {
            if(!keyboard.isKeyPressed(key))
            {
                pressed = false;
                return;
            }
        }
        if(!pressed){this_frame = true;}
        pressed = true;
    }
};

class KeyboardEvent{
    private:
    sf::Keyboard keyboard;
    public:
    sf::Keyboard::Key del = sf::Keyboard::Key::Backspace;
    sf::Keyboard::Key esc = sf::Keyboard::Key::Escape;
    sf::Keyboard::Key ctrl = sf::Keyboard::Key::LControl;
    sf::Keyboard::Key shift = sf::Keyboard::Key::LShift;
    bool shift_down = false;
    bool ctrl_down = false;
    bool esc_down = false;
    bool del_down = false;
    oneFrameKey save = {{ctrl, sf::Keyboard::Key::S}};
    oneFrameKey undo = {{ctrl, sf::Keyboard::Key::Z}};
    oneFrameKey redo = {{ctrl, sf::Keyboard::Key::Y}};
    void update()
    {
        save.Update(keyboard);
        undo.Update(keyboard);
        redo.Update(keyboard);
        ctrl_down = keyboard.isKeyPressed(ctrl);
        shift_down = keyboard.isKeyPressed(shift);
        esc_down = keyboard.isKeyPressed(esc);
        del_down = keyboard.isKeyPressed(del);
    }
};