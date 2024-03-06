#include "SFML/System/Vector2.hpp"
#include "game_entity.hpp"

bool isWithinBoundary(sf::Vector2f relative_pos, sf::Vector2f size)
{
    return relative_pos.x > 0 && relative_pos.y > 0 && relative_pos.x < size.x && relative_pos.y < size.y; 
}

bool intersectCircle(sf::Vector2f pos, sf::Vector2f origin, float radius)
{
    float x = pos.x - origin.x;
    float y = pos.y - origin.y;
    return x*x + y*y < radius*radius;
}

int mouseEntitiesIntersection(const sf::Vector2f& pos, const std::vector<entityBundle>& entities, const float& radius)
{

    for(int i = 0; i < entities.size(); i ++)
    {
        if (intersectCircle(pos, entities[i].position, radius)){return i;}
    }
    return -1;
}

sf::Vector2f vectorRescaler(sf::Vector2f pos, sf::Vector2f from_scale, sf::Vector2f to_scale)
{
    float x = pos.x / from_scale.x * to_scale.x;
    float y = pos.y / from_scale.y * to_scale.y;
    return {x, y};
}
