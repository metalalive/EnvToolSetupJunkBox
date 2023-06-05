-- the plug-in `packer` has to be manually installed first :
-- 
-- then add what other plug-ins based on your requirement to
-- following function block, once you've done, reload the script
-- by running `:luafile %`.
-- then update plug-ins by running `:PackerInstall`

return require('packer').startup(
function()
    -- Packer can manage itself
    use 'wbthomason/packer.nvim'
    -- `mason` is a Python-based package management tool used for only
    -- installing debugging tools like CodeLLDB, if you don't need it 
    -- you can keep it commented.
    --use 'williamboman/mason.nvim'
    --use 'williamboman/mason-lspconfig.nvim'
    -- Rust tools
    use 'neovim/nvim-lspconfig'
    use 'simrat39/rust-tools.nvim'
    -- Completion framework:
    use 'hrsh7th/nvim-cmp'
    -- LSP completion source:
    use 'hrsh7th/cmp-nvim-lsp'
    -- Useful completion sources:
    use 'hrsh7th/cmp-nvim-lua'
    use 'hrsh7th/cmp-nvim-lsp-signature-help'
    use 'hrsh7th/cmp-vsnip'
    use 'hrsh7th/cmp-path'
    use 'hrsh7th/cmp-buffer'
    use 'hrsh7th/vim-vsnip'
    -- work with other debuggers e.g. codelldb, vscode-tools, GDB
    use 'puremourning/vimspector'
end
)
