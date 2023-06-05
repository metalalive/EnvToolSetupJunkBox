--Set completeopt to have a better completion experience
vim.opt.completeopt = {'menuone', 'noselect', 'noinsert'}
vim.opt.shortmess = vim.opt.shortmess + {c = true}
vim.api.nvim_set_option('updatetime', 310)

-- Fixed column for diagnostics to appear
-- Show autodiagnostic popup on cursor hover_range
-- Goto previous / next diagnostic warning / error 
-- Show inlay_hints more frequently 
vim.cmd([[
    set signcolumn=yes
    autocmd CursorHold * lua vim.diagnostic.open_float(nil, { focusable = false })
]])

vim.cmd([[
    let g:vimspector_sidebar_width = 64
    let g:vimspector_bottombar_height = 15
    let g:vimspector_terminal_maxwidth = 60
    let g:vimspector_terminal_minwidth = 35
]]) -- the parameters for adjusting window layout are limited in `vimspector`

