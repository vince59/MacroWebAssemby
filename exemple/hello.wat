;; bonjour.wat (ordre corrigé)
(module
  ;; 1) TOUS les imports d'abord
  (import "env" "prompt" (func $prompt (param i32 i32) (result i32)))
  (import "env" "log"    (func $log    (param i32 i32)))

  ;; 2) Définitions locales ensuite
  (memory (export "memory") 1)
  (data (i32.const 0) "Bonjour, ")

  (func (export "run")
    (local $name_len i32)
    ;; lire le nom dans la mémoire à partir de l'offset 32
    i32.const 32
    i32.const 1024
    call $prompt
    local.set $name_len

    ;; "Bonjour, "
    i32.const 0
    i32.const 9
    call $log

    ;; le nom
    i32.const 32
    local.get $name_len
    call $log
  )
)
