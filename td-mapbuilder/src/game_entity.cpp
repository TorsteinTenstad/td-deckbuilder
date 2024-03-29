#pragma once
#include "SFML/Graphics/Color.hpp"
#include "SFML/System/Vector2.hpp"
#include <SFML/Graphics.hpp>
#include <cassert>
#include "game_entity.hpp"

void gameEntity::addEntity(sf::Vector2f position)
{
    entities.emplace_back(entityBundle{position, true, sf::CircleShape(radius)});
    entities.back().shape.setFillColor(fill_color);
    entities.back().shape.setOutlineColor(outline_color);
}
void gameEntity::reconstructEntityShapes()
{
    for(entityBundle& entity: entities)
    {
        entity.shape.setRadius(radius);
        entity.shape.setFillColor(fill_color);
        entity.shape.setOutlineColor(outline_color);
    }
}
void gameEntity::deleteEntity(int index)
{
    assert(entities.size() > index);
    entities.erase(entities.begin() + index);
}
void gameEntity::deleteSelectedEntities()
{
    entities.erase(std::remove_if(entities.begin(), entities.end(), [](entityBundle entity){return entity.is_selected;}), entities.end());
}
void gameEntity::deselectAll()
{
    for (auto& entity: entities)
    {
        entity.is_selected = false;
    }
}
void gameEntity::moveSelected(sf::Vector2f movement)
{
    for(auto& entity: entities)
    {
        if (entity.is_selected){entity.position+= movement;}
    }
}

