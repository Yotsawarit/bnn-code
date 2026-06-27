" BNN Code - Terminal-native AI coding agent powered by BNNs
" Maintainer: BNN Code Team <team@bnn-code.dev>
" Version: 0.3.0
" License: MIT
" URL: https://github.com/bnn-code/bnn-code

if exists('g:loaded_bnn')
    finish
endif
let g:loaded_bnn = 1

" Lazy-load on first command invocation
lua << EOF
vim.api.nvim_create_autocmd("VimEnter", {
    group = vim.api.nvim_create_augroup("bnn_plugin", { clear = true }),
    callback = function()
        require("bnn").setup()
    end,
    once = true,
})
EOF
