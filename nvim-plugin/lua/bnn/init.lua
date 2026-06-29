local M = {}

-- ============================================================================
-- Configuration
-- ============================================================================
M.config = {
    binary_path = "bnn",
    codebase_path = ".",
    show_output = true,
    streaming = true,
}

-- ============================================================================
-- Setup
-- ============================================================================
function M.setup(opts)
    M.config = vim.tbl_deep_extend("force", M.config, opts or {})

    -- Create user commands
    local commands = {
        { "BnnExplain",    M.explain,       {} },
        { "BnnRefactor",   M.refactor,      {} },
        { "BnnTest",       M.test,          {} },
        { "BnnFix",        M.fix,           {} },
        { "BnnFixCodebase", M.fix_codebase, {} },
        { "BnnCommit",     M.commit,        {} },
        { "BnnReview",     M.review,        {} },
        { "BnnDocument",   M.document,      {} },
        { "BnnQuery",      M.query,         { nargs = "?" } },
        { "BnnTerminal",   M.open_terminal, {} },
    }

    for _, cmd in ipairs(commands) do
        vim.api.nvim_create_user_command(cmd[1], cmd[2], cmd[3])
    end

    -- Create keymaps
    vim.keymap.set("v", "<leader>be", M.explain,    { desc = "BNN: Explain selection" })
    vim.keymap.set("v", "<leader>br", M.refactor,   { desc = "BNN: Refactor selection" })
    vim.keymap.set("v", "<leader>bt", M.test,       { desc = "BNN: Generate tests" })
    vim.keymap.set("v", "<leader>bd", M.document,   { desc = "BNN: Generate docs" })
    vim.keymap.set("n", "<leader>bf", M.fix,        { desc = "BNN: Fix current file" })
    vim.keymap.set("n", "<leader>bF", M.fix_codebase, { desc = "BNN: Fix entire codebase" })
    vim.keymap.set("n", "<leader>bc", M.commit,     { desc = "BNN: Commit message" })
    vim.keymap.set("n", "<leader>brv", M.review,    { desc = "BNN: Review changes" })
    vim.keymap.set("n", "<leader>bq", function()
        vim.ui.input({ prompt = "BNN Query: " }, function(input)
            if input and input ~= "" then
                M.query(input)
            end
        end)
    end, { desc = "BNN: Ask question" })
    vim.keymap.set("n", "<leader>bT", M.open_terminal, { desc = "BNN: Open terminal" })
end

-- ============================================================================
-- Internal helpers
-- ============================================================================

--- Execute BNN binary asynchronously
---@param args string[] command arguments
---@param callback function|nil called with output string on success
local function execute_bnn(args, callback)
    local cmd = M.config.binary_path .. " " .. table.concat(args, " ")

    vim.notify("🧠 BNN: Processing...", vim.log.levels.INFO)

    vim.fn.jobstart(cmd, {
        stdout_buffered = true,
        on_stdout = function(_, data)
            if data then
                local output = table.concat(data, "\n")
                if callback then
                    callback(output)
                else
                    M.show_output(output)
                end
            end
        end,
        on_stderr = function(_, data)
            if data and #data > 0 then
                local lines = vim.tbl_filter(function(l) return l ~= "" end, data)
                if #lines > 0 then
                    vim.notify("❌ BNN Error: " .. table.concat(lines, "\n"), vim.log.levels.ERROR)
                end
            end
        end,
        on_exit = function(_, code)
            if code == 0 then
                vim.notify("✅ BNN completed", vim.log.levels.INFO)
            else
                vim.notify("❌ BNN failed with code " .. code, vim.log.levels.ERROR)
            end
        end,
    })
end

--- Show output in a floating window
---@param output string
function M.show_output(output)
    if not M.config.show_output then
        return
    end

    -- Create buffer
    local buf = vim.api.nvim_create_buf(false, true)
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, vim.split(output, "\n"))

    -- Calculate window dimensions
    local width = math.min(math.floor(vim.o.columns * 0.85), 120)
    local height = math.min(math.floor(vim.o.lines * 0.8), 40)

    local opts = {
        relative = "editor",
        width = width,
        height = height,
        col = math.floor((vim.o.columns - width) / 2),
        row = math.floor((vim.o.lines - height) / 2),
        style = "minimal",
        border = "rounded",
        title = " 🧠 BNN Code ",
        title_pos = "center",
    }

    local win = vim.api.nvim_open_win(buf, true, opts)

    -- Keymaps to close
    vim.keymap.set("n", "q", "<cmd>close<CR>", { buffer = buf, noremap = true, silent = true })
    vim.keymap.set("n", "<Esc>", "<cmd>close<CR>", { buffer = buf, noremap = true, silent = true })

    -- Filetype for syntax highlighting
    vim.api.nvim_set_option_value("filetype", "markdown", { buf = buf })

    -- Auto-close on cursor last line + enter
    vim.keymap.set("n", "<CR>", function()
        local line = vim.fn.line(".")
        local last = vim.fn.line("$")
        if line == last then
            vim.api.nvim_win_close(win, true)
        end
    end, { buffer = buf, noremap = true, silent = true })
end

--- Get the current visual selection text
---@return string|nil
local function get_selected_text()
    local start_pos = vim.fn.getpos("'<")
    local end_pos = vim.fn.getpos("'>")
    local lines = vim.fn.getline(start_pos[2], end_pos[2])

    if #lines == 0 then
        return nil
    end

    -- Handle partial line selection
    if start_pos[2] == end_pos[2] then
        lines[1] = string.sub(lines[1], start_pos[3], end_pos[3])
    else
        lines[1] = string.sub(lines[1], start_pos[3])
        lines[#lines] = string.sub(lines[#lines], 1, end_pos[3])
    end

    return table.concat(lines, "\n")
end

--- Get current file path
---@return string
local function get_current_file()
    return vim.fn.expand("%:p")
end

-- ============================================================================
-- Commands
-- ============================================================================

function M.explain()
    local file = get_current_file()
    execute_bnn({ "explain", file, "--path", M.config.codebase_path })
end

function M.refactor()
    local file = get_current_file()
    execute_bnn({ "refactor", file, "--path", M.config.codebase_path })
end

function M.test()
    local file = get_current_file()
    execute_bnn({ "test", file, "--path", M.config.codebase_path })
end

function M.fix()
    local file = get_current_file()
    execute_bnn({ "fix", file, "--path", M.config.codebase_path })
end

function M.fix_codebase()
    execute_bnn({ "fix", "--path", M.config.codebase_path })
end

function M.commit()
    execute_bnn({ "commit", "--path", M.config.codebase_path }, function(output)
        -- Extract commit message from markdown code block or raw text
        local message = output:match("```.-\n(.-)\n```") or output

        -- Strip leading/trailing whitespace
        message = vim.trim(message)

        -- Show in input box for editing
        vim.ui.input({
            prompt = "Commit message: ",
            default = message,
        }, function(input)
            if input then
                vim.fn.setreg("+", input)
                vim.notify("📋 Commit message copied to clipboard!", vim.log.levels.INFO)
            end
        end)
    end)
end

function M.review()
    local file = get_current_file()
    execute_bnn({ "review", file, "--path", M.config.codebase_path })
end

function M.document()
    local file = get_current_file()
    execute_bnn({ "document", file, "--path", M.config.codebase_path })
end

function M.query(query)
    if not query or query == "" then
        vim.ui.input({ prompt = "BNN Query: " }, function(input)
            if input and input ~= "" then
                execute_bnn({ input, "--path", M.config.codebase_path })
            end
        end)
    else
        execute_bnn({ query, "--path", M.config.codebase_path })
    end
end

function M.open_terminal()
    vim.cmd("terminal " .. M.config.binary_path)
end

return M
