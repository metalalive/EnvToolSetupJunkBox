require('vars') -- Variables
require('opts') -- Options
require('keys') -- Keymaps
require('plug') -- Plugins

-- Mason Setup
-- require("mason").setup({
--     ui = {
--         icons = {
--             package_installed = "",
--             package_pending = "",
--             package_uninstalled = "",
--         },
--     }
-- })
-- 
-- require("mason-lspconfig").setup()

local rt = require("rust-tools")

-- CAUTION :
--   when the symbol is declared outside current Cargo project,
--   neovim seems to launch new `rust-analyzer` server for each
--   Cargo dependency, which is NOT memory-efficient.
--   e.g. 3 Cargo dependencies in your application, there will be 3
--   `rust-analyzer` server processes in your host machine.
--   These `rust-analyzer` servers will NOT terminate automatically
--   even after users go back from the symbol definition to their
--   original files.
--   TODO, switch to this alternative : https://github.com/pr2502/ra-multiplex
rt.setup({
  server = {
    on_attach = function(_, bufnr)
      -- Hover actions, show type / variable definition in a small popup
      vim.keymap.set("n", "<C-space>", rt.hover_actions.hover_actions, { buffer = bufnr })
      -- Expand macro, show macro definition
      vim.keymap.set("n", 'mac', rt.expand_macro.expand_macro, { buffer = bufnr })
    end,
    settings = {
        ['rust-analyzer'] = {
            cachePriming = {enable = false},
            lru = {capacity = 32}
        }
    }
  },
}) -- end of rust-tools setup

-- does the plug-in `lspconfig` blocks `rust-tools` ?
local default_lsp = require("lspconfig")
-- default_lsp.rust_analyzer.setup({
--     settings = {
-- 	["rust_analyzer"] = {
-- 	    cargo = {buildScripts = {enable = true}}
--         }
--     }
-- })
-- reference : https://rust-analyzer.github.io/manual.html#go-to-definition

default_lsp.pylsp.setup({
    settings = {
    pylsp = {
    plugins = {
	maxLineLength = 250,
	jedi_completion = {
	    include_class_objects = true,
	    include_function_objects = true
	},
	jedi = {
	    environment = os.getenv("VENV_PATH_PYLSP")
	} -- where OS env vars kick in
    }}}
})

-- key mapping for go-to definition.
-- https://neovim.io/doc/user/lsp.html#lsp-buf
vim.api.nvim_buf_set_keymap(0, 'n', '<C-]>',
  '<cmd>lua vim.lsp.buf.definition()<CR>',
  { noremap = true, silent = true })
-- get back from the definition, by the key combo below :
-- 1. Ctrl + o , see https://github.com/prabirshrestha/vim-lsp/issues/434
-- 2. Ctrl + t , it might be Ubuntu package `ctag` doing the trick.


-- Language Server Protocol Diagnostics Options Setup 
local diagno_sign_reg = function(opts)
    vim.fn.sign_define(opts.name, {
        texthl = opts.name,
        text = opts.text,
        numhl = ''
    })
end

diagno_sign_reg({name = 'DiagnosticSignError', text = "E"})
diagno_sign_reg({name = 'DiagnosticSignWarn', text = "W"})
diagno_sign_reg({name = 'DiagnosticSignHint', text = "H"})
diagno_sign_reg({name = 'DiagnosticSignInfo', text = "I"})

vim.diagnostic.config({
    virtual_text = false,
    signs = true,
    update_in_insert = true,
    underline = true,
    severity_sort = false,
    float = {
        border = 'rounded',
        source = 'always',
        header = '',
        prefix = '',
    },
})

vim.cmd([[
    set signcolumn=yes
    autocmd CursorHold * lua vim.diagnostic.open_float(nil, { focusable = false })
]])


-- Completion Plugin Setup
local cmp = require'cmp'
cmp.setup({
  -- Enable LSP snippets
  snippet = {
    expand = function(args)
        vim.fn["vsnip#anonymous"](args.body)
    end,
  },
  mapping = {
    ['<C-Up>'] = cmp.mapping.select_prev_item(),
    ['<C-Down>'] = cmp.mapping.select_next_item(),
    -- extra key combo to move between options
    ['<S-Tab>'] = cmp.mapping.select_prev_item(),
    ['<Tab>']   = cmp.mapping.select_next_item(),
    ['<S-Up>'] = cmp.mapping.scroll_docs(-3),
    ['<S-Down>'] = cmp.mapping.scroll_docs(3),
    ['<C-e>'] = cmp.mapping.complete(),
    ['<Esc>'] = cmp.mapping.close(),
    ['<CR>'] = cmp.mapping.confirm({
      behavior = cmp.ConfirmBehavior.Insert,
      select = true,
    })
  },
  -- Installed sources:
  sources = {
    { name = 'path' },                              -- file paths
    { name = 'nvim_lsp', keyword_length = 1 },      -- from language server
    { name = 'nvim_lsp_signature_help'},            -- display function signatures with current parameter emphasized
    { name = 'nvim_lua', keyword_length = 3},       -- complete neovim's Lua runtime API such vim.lsp.*
    { name = 'buffer', keyword_length = 2 },        -- source current buffer
    { name = 'vsnip', keyword_length = 2 },         -- nvim-cmp source for vim-vsnip 
    { name = 'calc'},                               -- source for math calculation
  },
  window = {
      completion = cmp.config.window.bordered(),
      documentation = cmp.config.window.bordered(),
  },
  formatting = {
      fields = {'menu', 'abbr', 'kind'},
      format = function(entry, item)
          local menu_icon ={
              nvim_lsp = 'λ',
              vsnip = '⋗',
              buffer = 'Ω',
              path = 'ρ',
          }
          item.menu = menu_icon[entry.source.name]
          return item
      end,
  },
})

