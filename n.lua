-- local config = {
--     name = "md-lsp",
--     cmd = { "./target/debug/md-lsp" },
--     root_dir = vim.loop.cwd(),
-- }
--
--
-- local id = vim.lsp.start_client(config)
--
-- local bufnr = vim.api.nvim_get_current_buf()
--
-- if not vim.lsp.buf_is_attached(bufnr, id) then
--     vim.lsp.buf_attach_client(bufnr, id)
-- end

local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

configs.md_lsp = {
    default_config = {
        name = "md-lsp",
        cmd = { "./target/debug/md-lsp" },
        -- cmd = { "md-lsp" },
        filetypes = { "markdown" },
        root_dir = lspconfig.util.root_pattern('.git'),
        single_file_support = true,
    },
}

lspconfig.md_lsp.setup({})
