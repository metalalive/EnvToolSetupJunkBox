-- Functional wrapper for mapping custom keybindings
-- mode (as in Vim modes like Normal/Insert mode)
-- lhs (the custom keybinds you need)
-- rhs (the commands or existing keybinds to customise)
-- opts (additional options like <silent>/<noremap>, see :h map-arguments for more info on it)
function map(mode, lhs, rhs, opts)
    local options = { noremap = true }
    if opts then
        options = vim.tbl_extend("force", options, opts)
    end
    vim.api.nvim_set_keymap(mode, lhs, rhs, options)
end

-- Vimspector 
-- Hot-keys for debugger
vim.cmd([[
    nmap <F5> <cmd>call vimspector#Launch()<cr>
    nmap <F8> <cmd>call vimspector#Reset()<cr>
    nmap <F9> <cmd>call vimspector#Continue()<cr>
    nmap <F10> <cmd>call vimspector#StepOver()<cr>
    nmap <F11> <cmd>call vimspector#StepOut()<cr>
    nmap <F12> <cmd>call vimspector#StepInto()<cr>
]])

map('n', "bp", ":call vimspector#ToggleBreakpoint()<cr>")
map('n', "wp", ":call vimspector#AddWatch()<cr>")
map('n', "ev", ":call vimspector#Evaluate()<cr>")

