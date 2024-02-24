#pragma once
#include "git2/checkout.h"
#include "git2/commit.h"
#include "git2/global.h"
#include "git2/index.h"
#include "git2/object.h"
#include "git2/oid.h"
#include "git2/refs.h"
#include "git2/repository.h"
#include "git2/reset.h"
#include "git2/signature.h"
#include "git2/sys/commit.h"
#include "git2/tree.h"
#include "git2/types.h"
#include <cassert>
#include <cstddef>
#include <iostream>
#include <git2.h>
#include <utility>
#include <vector>
#include "file_sys.cpp"

class gitHandler{
    public:
    std::string repository_path;
    std::vector<git_oid> commit_ids;
    int idx;

    gitHandler(std::string project_path) : repository_path(project_path)
    {
        // Initialize and push initial commit if project_path is not an existing repo. Commit_ids and idx are set by iterating backwards either from HEAD or cached id at 'project_path/oid.txt'.
        git_libgit2_init();
        const char* path = repository_path.c_str();
        git_repository* repo = nullptr;
        int error = git_repository_open(&repo,  path);

        if (error != 0) {
            // Directory is not a Git repository, initialize it
            error = git_repository_init(&repo, path, false);
            if (error != 0) {
                // Handle initialization error
                git_libgit2_shutdown();
                return;
            }
            // Create initial commit to new repository
            if(!initialCommit(repo)){return;}
        } 

        // Fill commit_ids and set the index of head
        git_oid head_id, end_id;
        if (git_reference_name_to_id(&head_id, repo, "HEAD") != 0){return;}
        if (findFileInDirectory(project_path, "oid", {"txt"}) != "")
        {
            end_id = loadCommitIdFromFile(project_path + "/oid.txt");
            git_commit* commit = nullptr;
            if(git_commit_lookup(&commit, repo, &end_id) != 0){end_id = head_id;}
            git_commit_free(commit);
        }
        else
        {
            end_id = head_id;
        }

        // List commits
        git_revwalk *walker = nullptr;
        git_revwalk_new(&walker, repo);
        git_revwalk_push(walker, &end_id);

        int i = 0;
        bool iterate = false;
        
        git_oid oid;
        while (!git_revwalk_next(&oid, walker)) {
            if(iterate){i += 1;}
            commit_ids.insert(commit_ids.begin(), oid);
            if (git_oid_equal(&head_id, &end_id)){iterate= true;}
        }
        idx = i;

        // Clean up
        git_revwalk_free(walker);
        git_repository_free(repo);
        git_libgit2_shutdown();
        return;
    }
    bool initialCommit(git_repository* repo){
        git_signature* sig = nullptr;
        git_index* index = nullptr;
        git_oid tree_id, commit_id;
        git_tree* tree = nullptr;
        if(git_signature_default(&sig, repo) < 0){
            assert(false);
        }
        if (git_repository_index(&index, repo) < 0)
            assert(false);

        if (git_index_write_tree(&tree_id, index) < 0)
            assert(false);

        git_index_free(index);
        if (git_tree_lookup(&tree, repo, &tree_id) < 0)
            assert(false);
        
        if (git_commit_create( &commit_id, repo, "HEAD", sig, sig,
                NULL, "Initial commit", tree, 0, NULL) < 0)
            assert(false);

        git_tree_free(tree);
        git_signature_free(sig);

        return true;
    }

    bool initializeRepository() {
        const char* path = repository_path.c_str();
        git_libgit2_init();

        git_repository* repo = nullptr;
        int error = git_repository_open(&repo, path);

        if (error != 0) {
            // Directory is not a Git repository, initialize it
            error = git_repository_init(&repo, path, false);
            if (error != 0) {
                // Handle initialization error
                git_libgit2_shutdown();
                return false;
            }
            // Repository initialized successfully

            // Create initial commit
            if(!initialCommit(repo))
                return false;
        }

        //std::cout << git_repository_path(repo) << "\n";
        
        // Clean up
        git_repository_free(repo);
        git_libgit2_shutdown();
        return true;
    }

    bool stageAndCommit(const std::vector<std::string> stage_files)
    {
        const char* path = repository_path.c_str();
        git_libgit2_init();

        git_repository* repo = nullptr;
        git_signature* sig = nullptr;
        git_index* index = nullptr;
        git_oid tree_id, commit_id, parent_id;
        git_commit* parent = nullptr;
        const git_commit* const_parent = nullptr;
        git_tree* tree = nullptr;
        if (git_repository_open(&repo, path) != 0){return false;}
        if (git_signature_default(&sig, repo) < 0){return false;}
        if (git_repository_index(&index, repo) < 0) {return false;}
        for(const std::string& file: stage_files)
        {
            if(git_index_add_bypath(index, file.c_str()) != 0){return false;}
        }
        if (git_index_write_tree(&tree_id, index) < 0){return false;}

        git_index_free(index);
        if (git_tree_lookup(&tree, repo, &tree_id) < 0){return false;}
        if (git_reference_name_to_id(&parent_id, repo, "HEAD") != 0){return false;}
        if (git_commit_lookup(&parent, repo, &parent_id) != 0){return false;}
        const_parent = parent;
        if (git_commit_create(&commit_id, repo, "HEAD", sig, sig, NULL, "Update entities", tree, 1, &const_parent) != 0){return false;}

        idx += 1;
        for(int i = idx; i < commit_ids.size(); i ++)
        {
            commit_ids.erase(commit_ids.begin() + idx);
        }
        commit_ids.push_back(commit_id);
        git_commit_free(parent);
        git_tree_free(tree);
        git_signature_free(sig);
        git_repository_free(repo);
        git_libgit2_shutdown();
        return true;
    }

    bool ListCommits()
    {
        const char* path = repository_path.c_str();
        git_libgit2_init();
        // Open a repository
        git_repository *repo = nullptr;
        int error = git_repository_open(&repo, path);

        if (error != 0) {
            const git_error *e = giterr_last();
            std::cerr << "Error: " << e->message << std::endl;
            git_libgit2_shutdown();
            return 1;
        }

        // List commits
        git_revwalk *walker = nullptr;
        git_revwalk_new(&walker, repo);
        git_revwalk_push_head(walker);

        git_oid oid;
        while (!git_revwalk_next(&oid, walker)) {
            char commit_id[GIT_OID_HEXSZ + 1];
            git_oid_tostr(commit_id, sizeof(commit_id), &oid);
            std::cout << "Commit: " << commit_id << std::endl;
        }
        std::cout << "\n";

        // Cleanup
        git_revwalk_free(walker);
        git_repository_free(repo);
        git_libgit2_shutdown();

        return 0;
    }

    bool LoadCommit(git_oid commit)
    {
        const char* path = repository_path.c_str();
        git_libgit2_init();

        git_repository* repo = nullptr;
        git_object* commit_object = nullptr;
        git_checkout_options options{GIT_CHECKOUT_OPTIONS_VERSION, GIT_CHECKOUT_SAFE};
        if (git_repository_open(&repo, path) != 0){return false;}
        if(git_object_lookup(&commit_object, repo, &commit, GIT_OBJECT_COMMIT) != 0){return false;}
        
    
        if(git_reset(repo, commit_object, GIT_RESET_HARD, &options)!= 0){return false;}

        git_object_free(commit_object);
        git_repository_free(repo);
        git_libgit2_shutdown();
        return true;
    }

    bool Undo(bool to_head)
    {
        if (idx == 0){return false;}
        if(LoadCommit(commit_ids[idx - !to_head]))
        {
            idx = idx - !to_head;
            return true;
        };
        return false;
    }

    bool Redo()
    {
        if(idx >= commit_ids.size() - 1){return false;}
        if(LoadCommit(commit_ids[idx + 1])){
            idx = idx + 1;
            return true;
        }
        return false;
    }
};