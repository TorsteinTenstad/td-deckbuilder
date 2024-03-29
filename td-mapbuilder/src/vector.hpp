#pragma once
#include "SFML/System/Vector2.hpp"
#include "game_entity.hpp"

bool isWithinBoundary(sf::Vector2f relative_pos, sf::Vector2f size);
bool intersectCircle(sf::Vector2f pos, sf::Vector2f origin, float radius);
bool intersectRectangle(sf::Vector2f pos, sf::Vector2f upper_left, sf::Vector2f lower_right);
int mouseEntitiesIntersection(const sf::Vector2f& pos, const std::vector<entityBundle>& entities, const float& radius);
sf::Vector2f vectorRescaler(sf::Vector2f pos, sf::Vector2f from_scale, sf::Vector2f to_scale);
inline bool any(const std::vector<bool> b);