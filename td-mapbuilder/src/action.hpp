#pragma once
#include "SFML/System/Vector2.hpp"
#include "game_entity.hpp"
#include "cursor_and_keys.hpp"
#include "vector.hpp"
#include <iostream>
#include <variant>


inline int numSelected(const std::vector<entityBundle> entities)
{
    int num = 0;
    for (auto entity: entities)
    {
        if(entity.is_selected){num += 1;}
    }
    return num;
}

struct select_drag
{ 
    std::vector<bool> selected_by_drag;

    void act(gameEntity& game_entity, const MouseEvent& m_event, const KeyboardEvent& kb_event, bool& has_unsaved_changes)
    {
        if(game_entity.entities.size() == 0 || !m_event.moved_while_pressed){return;}
        
        if(selected_by_drag.size() == 0)
        {
            for (const auto& entity: game_entity.entities)
            {
                selected_by_drag.emplace_back(false);
            }
        }
        sf::Vector2f upper_left = sf::Vector2f(std::min(m_event.position.x, m_event.click_pos.x), std::min(m_event.position.y, m_event.click_pos.y));
        sf::Vector2f lower_right = sf::Vector2f(std::max(m_event.position.x, m_event.click_pos.x), std::max(m_event.position.y, m_event.click_pos.y));
        for(int i = 0; i < game_entity.entities.size(); i ++)
        {
            entityBundle& entity = game_entity.entities[i];
            bool intersect = intersectRectangle(entity.position, upper_left, lower_right);
            if (intersect)
            {
                selected_by_drag[i] = true;
                has_unsaved_changes = true;
            }
            if (selected_by_drag[i]){entity.is_selected = intersect;}
        }
        if(m_event.released_this_frame){selected_by_drag.clear();}
    }
};

struct select_click{
    int select_id = -1;
    void act(gameEntity& game_entity, const MouseEvent& m_event, const KeyboardEvent& kb_event, bool& has_unsaved_changes)
    {
        if(game_entity.entities.size() == 0){return;}
        int intersect_id = mouseEntitiesIntersection(m_event.position, game_entity.entities, game_entity.radius);

        /* 
        Implementation a bit unclear, because this works under the assumption that during releasethisframe, everything might or might not 
        deselect automatically. Also, select should be added during pressedthisframe, to facilitate easy select-and-move shenanigans.
        */
        if(m_event.pressed_this_frame && intersect_id >= 0)
        {
            !game_entity.entities[intersect_id].is_selected ? select_id = intersect_id : select_id = -1;
            game_entity.entities[intersect_id].is_selected = true;
        }
        if(m_event.released_this_frame && intersect_id >= 0)
        {
            game_entity.entities[intersect_id].is_selected = true;
            if(intersect_id != select_id)
            {
                game_entity.entities[intersect_id].is_selected = false;
                has_unsaved_changes = true;
            }
        }
    }
};

struct move
{
    void act(gameEntity& game_entity, const MouseEvent& m_event, const KeyboardEvent& kb_event, bool& has_unsaved_changes)
    {
        if (m_event.pressed)
        {
            game_entity.moveSelected(m_event.cursor_movement);
            has_unsaved_changes = true;
        }
    }
};

struct add
{
    void act(gameEntity& game_entity, const MouseEvent& m_event, const KeyboardEvent& kb_event, bool& has_unsaved_changes)
    {
        if (m_event.released_this_frame)
        {
            game_entity.addEntity(m_event.position);
            has_unsaved_changes = true;
        }
    }
};

struct del
{
    void act(gameEntity& game_entity, const MouseEvent& m_event, const KeyboardEvent& kb_event, bool& has_unsaved_changes)
    {
        game_entity.deleteSelectedEntities();
        has_unsaved_changes = true;
    }
};

struct none
{
    void act(gameEntity& game_entity, const MouseEvent& m_event, const KeyboardEvent& kb_event, bool& has_unsaved_changes)
    {
        return;
    }
};

using Mode = std::variant<select_click, select_drag, add, del, move, none>;

struct ActionOptions
{
    Mode mode = none{};
    bool has_unsaved_changes = false;
    bool will_save = false;
    bool will_undo = false;
    bool will_redo = false;
    bool attempt_global_deselect = false;
};

inline void executeAction(gameEntity& game_entity, const KeyboardEvent& kb_event, const MouseEvent& m_event, ActionOptions& action_opts)
{
    if(action_opts.attempt_global_deselect && (!kb_event.shift_down || kb_event.esc_down)){game_entity.deselectAll();}
    std::visit([&](auto& mode){mode.act(game_entity, m_event, kb_event, action_opts.has_unsaved_changes);}, action_opts.mode);
}

class ActionMode
{
    public:
    bool mass_select = false;
    bool moving = false;
    bool pressed_with_control = false;

    void compute(const gameEntity& game_entity, const KeyboardEvent& kb_event, const MouseEvent& m_event, ActionOptions& action_opts)
    {
        int num_selected = numSelected(game_entity.entities);
        int intersect_id = mouseEntitiesIntersection(m_event.position, game_entity.entities, game_entity.radius);
        
        bool in_select = std::holds_alternative<select_drag>(action_opts.mode) || std::holds_alternative<select_click>(action_opts.mode);
        
        action_opts.attempt_global_deselect = false;
        action_opts.will_save = false;
        action_opts.will_undo = false;
        action_opts.will_redo = false;
        
        // 1. If control is pressed and mouse left click pressed this frame, game will be in add mode for duration of press. 
        // Nothing will change until release, when a new entity will be added. Release will always cause commit.
        if(kb_event.ctrl_down && m_event.pressed_this_frame)
        {
            pressed_with_control = true;
        }
        if(pressed_with_control)
        {
            action_opts.mode = add{};
            if(m_event.released_this_frame)
            {
                action_opts.attempt_global_deselect = true;
                pressed_with_control = false;
                action_opts.will_save = true;
            }
            return;
        }
         
        /* 2. Implicit: control is not pressed. When pressed this frame, 
        if and only if the mouse is hovering an unselected entity will this yield 
        a definite mode: selecting said entity. Will not always lead to commit.*/
        if(m_event.pressed_this_frame && intersect_id >= 0 && !game_entity.entities[intersect_id].is_selected)
        {
            if(!std::holds_alternative<select_click>(action_opts.mode)){action_opts.mode = select_click{};}
            if(!in_select){action_opts.will_save = true;}
            
            action_opts.attempt_global_deselect = true;
            return;
        }

        /* 3. Moving while pressed will always give a definite mode. If either nothing is selected or shift_down,
        this will lead to mass select and select mode. This rule will also be true on the frame of release. 
        Assumes that changing num_selected is impossible during pressed.
        (TODO: make a minimal movement amount (>> 0) so this isn't activated spuriously). */
        if(m_event.moved_while_pressed)
        {
            assert(!(mass_select && moving));
            if (!(mass_select || moving))
            {
                (intersect_id < 0 || num_selected == 0 || kb_event.shift_down) ? mass_select = true: moving = true;
            }

            if(mass_select)
            {
                if(!std::holds_alternative<select_drag>(action_opts.mode)){action_opts.mode = select_drag{};}
                if(!in_select){action_opts.will_save = true;}
            }
            else
            {
                if(!std::holds_alternative<move>(action_opts.mode))
                {
                    action_opts.will_save = true;
                    action_opts.mode = move{};
                }
            }
            return;
        }

        mass_select = false;
        moving = false;


        /* 4. Implicit: Did not move while pressed, and is now released. Intersection will determine whether to add or toggle select,
        and not pressing shift will deselect everything else.*/
        if(m_event.released_this_frame)
        {
            action_opts.attempt_global_deselect = true;
            if(intersect_id >= 0 || num_selected > 0)
            {
                if(!std::holds_alternative<select_click>(action_opts.mode)){action_opts.mode = select_click{};}
                if(!in_select){action_opts.will_save = true;}
            }
            else 
            {
                action_opts.mode = add{};
                action_opts.will_save = true;
            }
            return;
        }

        /* 5. Keyboard shortcuts. Nothing happens if mouse button is pressed (to avoid deleting or unselecting during other actions).
        Apart from that, this should be pretty straight forward (?). Also controls undo/redo and manual save. */
        if(!m_event.pressed)
        {
            if((kb_event.save.this_frame || kb_event.undo.this_frame) && action_opts.has_unsaved_changes)
            {
                action_opts.will_save = true;
            }
            if(kb_event.undo.this_frame)
            {
                action_opts.will_undo = true;
            }
            else if (kb_event.redo.this_frame) {
                action_opts.will_redo = true;
            }
            else if(kb_event.del_down)
            {
                action_opts.mode = del{};
                if(num_selected > 0){action_opts.will_save = true;}
            }
            else if(kb_event.esc_down && num_selected > 0)
            {
                action_opts.attempt_global_deselect = true;
                if(!in_select){action_opts.will_save = true;}
                if(!std::holds_alternative<select_click>(action_opts.mode)){action_opts.mode = select_click{};}
            }
            return;
        }

        // 6. Either not sufficient input or no input at all; the system will remain in it's previous mode.
        return;
    }
};