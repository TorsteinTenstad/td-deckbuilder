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
#include <vector>
#include "file_sys.hpp"

class gitHandler{
    private:
    bool initializeRepository(git_repository* repo, int r_value);
    
    public:
    std::string repository_path;
    std::vector<git_oid> commit_ids;
    int idx;

    gitHandler(std::string project_path);
    
    bool gitInitWrapper(std::function<bool(git_repository* repo, int r_value)> func);
    bool initialCommit(git_repository* repo);
    bool stageAndCommit(const std::vector<std::string> stage_files);
    bool ListCommits();
    bool LoadCommit(git_oid commit);
    bool Undo();
    bool Redo();
};