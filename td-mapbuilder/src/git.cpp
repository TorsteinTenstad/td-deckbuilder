#include "git2/commit.h"
#include "git2/global.h"
#include "git2/signature.h"
#include "git2/types.h"
#include <cassert>
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