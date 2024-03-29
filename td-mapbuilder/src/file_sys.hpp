#pragma once
#include "SFML/Graphics/Color.hpp"
#include "SFML/System/Vector2.hpp"
#include "game_entity.hpp"
#include "git2/oid.h"
#include "json.hpp"
#include "nlohmann/adl_serializer.hpp"
#include "nlohmann/json_fwd.hpp"

using json = nlohmann::json;

template<> struct nlohmann::adl_serializer<sf::Vector2f>
{
    static void to_json(json& j, const sf::Vector2f& vector)
    {
        j = json{{"x", vector.x}, {"y", vector.y}};
    }
    static void from_json(const json& j, sf::Vector2f& vector)
    {
        j.at("x").get_to(vector.x);
        j.at("y").get_to(vector.y);
    }
};

template<> struct nlohmann::adl_serializer<sf::Color>
{
    static void to_json(json& j, const sf::Color& color)
    {
        j = json{{"a", color.a}, {"b", color.b}, {"g",color.g}, {"r", color.r}};
    }
    static void from_json(const json& j, sf::Color& color)
    {
        j.at("a").get_to(color.a);
        j.at("b").get_to(color.b);
        j.at("g").get_to(color.g);
        j.at("r").get_to(color.r);
    }
};
template<> struct nlohmann::adl_serializer<entityBundle>
{
    static void to_json(json& j, const entityBundle& entity)
    {
        j = json{{"position", entity.position}, {"is_selected", entity.is_selected}};
    }
    static void from_json(const json& j, entityBundle& entity)
    {
        j.at("position").get_to(entity.position);
        j.at("is_selected").get_to(entity.is_selected);
    }
};

template<> struct nlohmann::adl_serializer<gameEntity>
{
    static void to_json(json& j, const gameEntity& game_entity)
    {
        j = json{{"radius", game_entity.radius}, {"fill_color", game_entity.fill_color}, {"outline_color", game_entity.outline_color}, {"entities", game_entity.entities}};
    }
    static void from_json(const json& j, gameEntity& game_entity)
    {
        j.at("radius").get_to(game_entity.radius);
        j.at("fill_color").get_to(game_entity.fill_color);
        j.at("outline_color").get_to(game_entity.outline_color);
        j.at("entities").get_to(game_entity.entities);
    }
};

void listSubdirectories(const std::string& folderPath);
std::string findFileInDirectory(const std::string& projectPath, const std::string& fileName, const std::vector<std::string>& file_types);
std::string findSubDirectory(const std::string& dir, const std::string& subdir);

std::string createNewProject(std::string project_folder);

void saveEntitiesToFile(std::string filename, const gameEntity& game_entity);
void saveCommitIdToFile(std::string filename, git_oid oid);
gameEntity loadEntitiesFromFile(std::string filename);
git_oid loadCommitIdFromFile(std::string filename);
