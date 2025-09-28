(module
  (import "gaufre" "invoke" (func $invoke (param i32 i32 i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "console.log")
  (data (i32.const 16) "[\"Bonjour de Gaufre!\"]")
  (data (i32.const 48) "[\"ligne 1\"]")
  (func (export "main")
    i32.const 0      ;; name ptr: "console.log"
    i32.const 11
    i32.const 16  ;; args ptr: ["..."] (JSON)
    i32.const 22
    i32.const 4096   ;; ret buf ptr
    i32.const 1024   ;; ret cap
    call $invoke
    drop             ;; ignore le retour (console.log -> null)
    i32.const 0      ;; name ptr: "console.log"
    i32.const 11
    i32.const 48  ;; args ptr: ["..."] (JSON)
    i32.const 11
    i32.const 4096   ;; ret buf ptr
    i32.const 1024   ;; ret cap
    call $invoke
    drop             ;; ignore le retour (console.log -> null)
  )
)
