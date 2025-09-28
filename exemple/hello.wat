(module
  (import "gaufre" "invoke" (func $invoke (param i32 i32 i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "console.log")
  (data (i32.const 16) "\"Bonjour de Gaufre!\"")
  (data (i32.const 48) "\"ligne \"")

  ;; i32_to_json(val, dst) -> len
  (func $i32_to_json (param $v i32) (param $dst i32) (result i32)
    (local $neg i32) (local $pos i32) (local $d i32) (local $i i32) (local $j i32) (local $t i32)
    i32.const 0
    local.set $neg
    local.get $v
    i32.const 0
    i32.lt_s
    if
      i32.const 1
      local.set $neg
      i32.const 0
      local.get $v
      i32.sub
      local.set $v
    end
    ;; v==0 -> "0"
    local.get $v
    i32.eqz
    if
      local.get $dst
      i32.const 48
      i32.store8
      i32.const 1
      return
    end
    i32.const 0
    local.set $pos
    block $digits_exit
      loop $digits
        local.get $v
        i32.const 10
        i32.rem_u
        local.set $d
        local.get $dst
        local.get $pos
        i32.add
        local.get $d
        i32.const 48
        i32.add
        i32.store8
        local.get $pos
        i32.const 1
        i32.add
        local.set $pos

        local.get $v
        i32.const 10
        i32.div_u
        local.set $v

        local.get $v
        i32.eqz
        br_if $digits_exit
        br $digits
      end
    end
    ;; ajoute '-' si négatif
    local.get $neg
    if
      local.get $dst
      local.get $pos
      i32.add
      i32.const 45
      i32.store8
      local.get $pos
      i32.const 1
      i32.add
      local.set $pos
    end
    ;; reverse in place [0..pos-1]
    i32.const 0
    local.set $i
    local.get $pos
    i32.const 1
    i32.sub
    local.set $j
    block $rev_exit
      loop $rev
        local.get $i
        local.get $j
        i32.ge_u
        br_if $rev_exit

        local.get $dst
        local.get $i
        i32.add
        i32.load8_u
        local.set $t

        local.get $dst
        local.get $i
        i32.add
        local.get $dst
        local.get $j
        i32.add
        i32.load8_u
        i32.store8

        local.get $dst
        local.get $j
        i32.add
        local.get $t
        i32.store8

        local.get $i
        i32.const 1
        i32.add
        local.set $i

        local.get $j
        i32.const 1
        i32.sub
        local.set $j

        br $rev
      end
    end
    local.get $pos
  )
  (func (export "main")
    (local $pos i32)
    (local $i i32)
    i32.const 512
    i32.const 91  ;; '['
    i32.store8
    i32.const 1
    local.set $pos
    ;; copie string JSON
    i32.const 512
    local.get $pos
    i32.add
    i32.const 16
    i32.const 20
    memory.copy
    local.get $pos
    i32.const 20
    i32.add
    local.set $pos
    i32.const 512
    local.get $pos
    i32.add
    i32.const 93  ;; ']'
    i32.store8
    local.get $pos
    i32.const 1
    i32.add
    local.set $pos
    i32.const 0      ;; name: "console.log"
    i32.const 11
    i32.const 512   ;; args ptr
    local.get $pos  ;; args len
    i32.const 4096  ;; ret ptr
    i32.const 1024  ;; ret cap
    call $invoke
    drop
    ;; for i = 1 to 10
    i32.const 1
    local.set $i
    block $exit
    loop $loop
    local.get $i
    i32.const 10
    i32.gt_s
    br_if $exit
    i32.const 512
    i32.const 91  ;; '['
    i32.store8
    i32.const 1
    local.set $pos
    ;; copie string JSON
    i32.const 512
    local.get $pos
    i32.add
    i32.const 48
    i32.const 8
    memory.copy
    local.get $pos
    i32.const 8
    i32.add
    local.set $pos
    i32.const 512
    local.get $pos
    i32.add
    i32.const 44  ;; ','
    i32.store8
    local.get $pos
    i32.const 1
    i32.add
    local.set $pos
    ;; var i -> JSON
    local.get $i
    i32.const 512
    local.get $pos
    i32.add
    call $i32_to_json
    local.get $pos
    i32.add
    local.set $pos
    i32.const 512
    local.get $pos
    i32.add
    i32.const 93  ;; ']'
    i32.store8
    local.get $pos
    i32.const 1
    i32.add
    local.set $pos
    i32.const 0      ;; name: "console.log"
    i32.const 11
    i32.const 512   ;; args ptr
    local.get $pos  ;; args len
    i32.const 4096  ;; ret ptr
    i32.const 1024  ;; ret cap
    call $invoke
    drop
    local.get $i
    i32.const 1
    i32.add
    local.set $i
    br $loop
    end
    end
  )
)
