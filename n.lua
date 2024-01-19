local id = vim.lsp.start_client({
    name = "md-lsp",
    cmd = {"./target/debug/md-lsp"},
})

local bufnr = vim.api.nvim_get_current_buf()

if not vim.lsp.buf_is_attached(bufnr, id) then
    vim.lsp.buf_attach_client(bufnr, id)
end

-- local function attach_lsp(args)
--     if id == nil then
--         return
--     end
--
--     local bufnr = args.buffer or args.buf;
--     if not bufnr or not filter(bufnr) then
--         return;
--     end
--
--     if not vim.lsp.buf_is_attached(args.buffer, id) then
--         vim.lsp.buf_attach_client(args.buffer, id);
--     end
-- end
