#pragma once
#include "git2/checkout.h"
#include "git2/commit.h"
#include "git2/global.h"
#include "git2/index.h"
#include "git2/object.h"
#include "git2/refs.h"
#include "git2/repository.h"
#include "git2/reset.h"
#include "git2/revert.h"
#include "git2/signature.h"
#include "git2/tree.h"
#include "git2/types.h"
#include <cassert>
#include <cstddef>
#include <iostream>
#include <git2.h>

bool gitInitialCommit(git_repository* repo){
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

bool initializeGitRepository(std::string& path) {
    git_libgit2_init();

    const char* c_path = path.c_str();
    git_repository* repo = nullptr;
    int error = git_repository_open(&repo, c_path);

    if (error != 0) {
        // Directory is not a Git repository, initialize it
        error = git_repository_init(&repo, c_path, false);
        if (error != 0) {
            // Handle initialization error
            git_libgit2_shutdown();
            return false;
        }
        // Repository initialized successfully

        // Create initial commit
        if(!gitInitialCommit(repo))
            return false;
    }

    //std::cout << git_repository_path(repo) << "\n";
    
    // Clean up
    git_repository_free(repo);
    git_libgit2_shutdown();
    return true;
}

bool gitStageAndCommit(std::string project, std::string stage_file)
{
    const char* c_project_path = project.c_str();
    const char* c_file= stage_file.c_str();
    git_libgit2_init();

    git_repository* repo = nullptr;
    git_signature* sig = nullptr;
    git_index* index = nullptr;
    git_oid tree_id, commit_id, parent_id;
    git_commit* parent = nullptr;
    const git_commit* const_parent = nullptr;
    git_tree* tree = nullptr;
    if (git_repository_open(&repo, c_project_path) != 0){return false;}
    if (git_signature_default(&sig, repo) < 0){return false;}
    if (git_repository_index(&index, repo) < 0) {return false;}
    if(git_index_add_bypath(index, c_file) != 0){return false;}
	if (git_index_write_tree(&tree_id, index) < 0){return false;}

	git_index_free(index);
    if (git_tree_lookup(&tree, repo, &tree_id) < 0){return false;}
    if (git_reference_name_to_id(&parent_id, repo, "HEAD") != 0){return false;}
    if (git_commit_lookup(&parent, repo, &parent_id) != 0){return false;}
    const_parent = parent;
    if (git_commit_create(&commit_id, repo, "HEAD", sig, sig, NULL, "Update entities", tree, 1, &const_parent) != 0){return false;}

    git_commit_free(parent);
    git_tree_free(tree);
	git_signature_free(sig);
    git_repository_free(repo);
    git_libgit2_shutdown();
    return true;
}

bool gitListCommits(std::string project_path)
{
 git_libgit2_init();

    // Open a repository
    git_repository *repo = nullptr;
    int error = git_repository_open(&repo, project_path.c_str());

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

bool gitUndo(std::string project_path, bool to_head)
{
    const char* c_project_path = project_path.c_str();
    git_libgit2_init();

    git_repository* repo = nullptr;
    git_signature* sig = nullptr;
    git_index* index = nullptr;
    git_oid tree_id, temp_id, parent_id;
    git_commit* temp_commit = nullptr;
    git_object* commit_object = nullptr;
    git_tree* tree = nullptr;
    if (git_repository_open(&repo, c_project_path) != 0){return false;}
    if (git_signature_default(&sig, repo) < 0){return false;}
    if (git_repository_index(&index, repo) < 0) {return false;}
	if (git_index_write_tree(&tree_id, index) < 0){return false;}

	git_index_free(index);
    if (git_tree_lookup(&tree, repo, &tree_id) < 0){return false;}
    if (git_reference_name_to_id(&temp_id, repo, "HEAD") != 0){return false;}
    if (to_head)
    {
        parent_id = temp_id;
    }
    else
    {
        if(git_commit_lookup(&temp_commit, repo, &temp_id) != 0){return false;}
        if(git_commit_parentcount(temp_commit) == 0){return false;}
        parent_id = *git_commit_parent_id(temp_commit, 0);
    }
    if(git_object_lookup(&commit_object, repo, &parent_id, GIT_OBJECT_COMMIT) != 0){return false;}
    
    git_checkout_options options{GIT_CHECKOUT_OPTIONS_VERSION, GIT_CHECKOUT_SAFE};
 
    if(git_reset(repo, commit_object, GIT_RESET_HARD, &options)!= 0){return false;}

    git_commit_free(temp_commit);
    git_object_free(commit_object);
    git_tree_free(tree);
	git_signature_free(sig);
    git_repository_free(repo);
    git_libgit2_shutdown();
    return true;
}