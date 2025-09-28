(module
  ;; --- IMPORTS d'abord ---
  (import "env" "invoke" (func $invoke (param i32 i32 i32 i32 i32 i32) (result i32)))
  (import "env" "log"    (func $log    (param i32 i32)))

  ;; --- Mémoire exportée ---
  (memory (export "memory") 1)

  ;; --- Littéraux en mémoire ---
  ;; noms de fonctions
  (data (i32.const 0)   "Math.sin")      ;; len = 8
  (data (i32.const 16)  "Math.cos")      ;; len = 8
  (data (i32.const 32)  "Math.tan")      ;; len = 8
  (data (i32.const 48)  "Math.random")   ;; len = 11

  ;; arguments JSON
  (data (i32.const 64)  "[1]")           ;; len = 3   (utilisé pour sin/cos/tan)
  (data (i32.const 80)  "[]")            ;; len = 2   (utilisé pour random)

  ;; étiquettes d'affichage
  (data (i32.const 96)  "sin=")          ;; len = 4
  (data (i32.const 100) "cos=")          ;; len = 4
  (data (i32.const 104) "tan=")          ;; len = 4
  (data (i32.const 108) "random=")       ;; len = 7
  (data (i32.const 200) "\n")            ;; len = 1

  ;; --- Programme ---
  (func (export "run")
    (local $n i32)            ;; longueur écrite dans le buffer de retour
    (local $ret i32)          ;; ptr buffer retour (constant 256, mais on le met en local si tu veux changer)

    ;; pointeur du buffer de retour
    i32.const 256
    local.set $ret

    ;; === sin(1) ===
    i32.const 96              ;; "sin="
    i32.const 4
    call $log

    i32.const 0               ;; name: "Math.sin"
    i32.const 8
    i32.const 64              ;; args: "[1]"
    i32.const 3
    local.get $ret            ;; ret ptr
    i32.const 128             ;; ret cap
    call $invoke
    local.set $n

    local.get $ret
    local.get $n
    call $log

    i32.const 200             ;; "\n"
    i32.const 1
    call $log

    ;; === cos(1) ===
    i32.const 100             ;; "cos="
    i32.const 4
    call $log

    i32.const 16              ;; "Math.cos"
    i32.const 8
    i32.const 64              ;; "[1]"
    i32.const 3
    local.get $ret
    i32.const 128
    call $invoke
    local.set $n

    local.get $ret
    local.get $n
    call $log

    i32.const 200
    i32.const 1
    call $log

    ;; === tan(1) ===
    i32.const 104             ;; "tan="
    i32.const 4
    call $log

    i32.const 32              ;; "Math.tan"
    i32.const 8
    i32.const 64              ;; "[1]"
    i32.const 3
    local.get $ret
    i32.const 128
    call $invoke
    local.set $n

    local.get $ret
    local.get $n
    call $log

    i32.const 200
    i32.const 1
    call $log

    ;; === Math.random() ===
    i32.const 108             ;; "random="
    i32.const 7
    call $log

    i32.const 48              ;; "Math.random"
    i32.const 11
    i32.const 80              ;; "[]"
    i32.const 2
    local.get $ret
    i32.const 128
    call $invoke
    local.set $n

    local.get $ret
    local.get $n
    call $log

    i32.const 200
    i32.const 1
    call $log
  )
)
