use crate::parser::{Program, Stmt, Expr};
use std::collections::BTreeMap;

/// échappement pour littéral WAT
fn wat_escape(s: &str) -> String {
    let mut out = String::new();
    for &b in s.as_bytes() {
        match b {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7E => out.push(b as char),
            _ => out.push_str(&format!("\\{:02x}", b)),
        }
    }
    out
}

pub fn generate_wat(prog: &Program) -> String {
    // --- segments de données ---
    let mut data = String::new();
    // "console.log" à 0
    data.push_str(&format!("  (data (i32.const 0) \"{}\")\n", wat_escape("console.log")));

    // Intern des chaînes JSON "\"...\"" avec dédup
    let mut str_off: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut next_off: usize = 16;

    let mut intern_string = |s: &str| -> (usize, usize) {
        let json = serde_json::to_string(s).unwrap(); // "\"...\""
        if let Some(&(off, len)) = str_off.get(&json) {
            return (off, len);
        }
        let len = json.as_bytes().len();
        data.push_str(&format!(
            "  (data (i32.const {off}) \"{lit}\")\n",
            off = next_off,
            lit = wat_escape(&json)
        ));
        str_off.insert(json, (next_off, len));
        next_off += len;
        if next_off % 16 != 0 {
            next_off += 16 - (next_off % 16);
        }
        (next_off - len, len)
    };

    // Pré-intern toutes les chaînes utilisées dans le programme
    fn walk_strings<'a>(ss: &'a [Stmt], out: &mut Vec<&'a str>) {
        for s in ss {
            match s {
                Stmt::Log(args) => {
                    for a in args {
                        if let Expr::Str(t) = a {
                            out.push(t);
                        }
                    }
                }
                Stmt::For { body, .. } => walk_strings(body, out),
            }
        }
    }
    let mut strings = Vec::new();
    walk_strings(&prog.stmts, &mut strings);
    for s in strings {
        let _ = intern_string(s);
    }

    // Collecte des variables i32 (noms rencontrés dans les for)
    fn collect_vars<'a>(ss: &'a [Stmt], acc: &mut BTreeMap<&'a str, ()>) {
        for s in ss {
            if let Stmt::For { name, body, .. } = s {
                acc.insert(name.as_str(), ());
                collect_vars(body, acc);
            }
        }
    }
    let mut vars = BTreeMap::<&str, ()>::new();
    collect_vars(&prog.stmts, &mut vars);

    // --- Génération du corps de main ---
    let mut body = String::new();

    // helper: émet un log avec args multiples -> construit JSON array dans 512
    let mut emit_log = |args: &[Expr], body: &mut String| {
        // '['
        body.push_str("    i32.const 512\n    i32.const 91  ;; '['\n    i32.store8\n");
        // pos = 1
        body.push_str("    i32.const 1\n    local.set $pos\n");

        for (k, e) in args.iter().enumerate() {
            // virgule si pas premier
            if k > 0 {
                body.push_str(
                    "    i32.const 512\n    local.get $pos\n    i32.add\n    i32.const 44  ;; ','\n    i32.store8\n",
                );
                body.push_str("    local.get $pos\n    i32.const 1\n    i32.add\n    local.set $pos\n");
            }

            match e {
                Expr::Str(s) => {
                    let (off, len) = intern_string(s);
                    body.push_str(&format!(
                        "    ;; copie string JSON\n    i32.const 512\n    local.get $pos\n    i32.add\n    i32.const {src}\n    i32.const {len}\n    memory.copy\n",
                        src = off,
                        len = len
                    ));
                    body.push_str("    local.get $pos\n    i32.const ");
                    body.push_str(&format!("{}\n", len));
                    body.push_str("    i32.add\n    local.set $pos\n");
                }
                Expr::Var(name) => {
                    body.push_str(&format!(
                        concat!(
                            "    ;; var {n} -> JSON\n",
                            "    local.get ${n}\n",
                            "    i32.const 512\n",
                            "    local.get $pos\n",
                            "    i32.add\n",
                            "    call $i32_to_json\n", // retourne len
                            "    local.get $pos\n",
                            "    i32.add\n",
                            "    local.set $pos\n"
                        ),
                        n = name
                    ));
                }
                Expr::Int(v) => {
                    body.push_str(&format!(
                        concat!(
                            "    ;; int {v}\n",
                            "    i32.const {v}\n",
                            "    i32.const 512\n",
                            "    local.get $pos\n",
                            "    i32.add\n",
                            "    call $i32_to_json\n",
                            "    local.get $pos\n",
                            "    i32.add\n",
                            "    local.set $pos\n"
                        ),
                        v = v
                    ));
                }
            }
        }

        // ']' et longueur totale = pos + 1
        body.push_str(
            "    i32.const 512\n    local.get $pos\n    i32.add\n    i32.const 93  ;; ']'\n    i32.store8\n",
        );
        body.push_str("    local.get $pos\n    i32.const 1\n    i32.add\n    local.set $pos\n");

        // invoke console.log(["…", …])
        body.push_str(
            concat!(
                "    i32.const 0      ;; name: \"console.log\"\n",
                "    i32.const 11\n",
                "    i32.const 512   ;; args ptr\n",
                "    local.get $pos  ;; args len\n",
                "    i32.const 4096  ;; ret ptr\n",
                "    i32.const 1024  ;; ret cap\n",
                "    call $invoke\n",
                "    drop\n",
            ),
        );
    };

    // émet un for i=start..end (inclus) avec body
    fn emit_for(
        name: &str,
        start: i32,
        end: i32,
        inner: &[Stmt],
        body: &mut String,
        emit_log: &mut dyn FnMut(&[Expr], &mut String),
    ) {
        body.push_str(&format!(
            "    ;; for {n} = {a} to {b}\n    i32.const {a}\n    local.set ${n}\n",
            n = name,
            a = start,
            b = end
        ));
        body.push_str("    block $exit\n    loop $loop\n");
        // break si i > end
        body.push_str(&format!(
            "    local.get ${n}\n    i32.const {b}\n    i32.gt_s\n    br_if $exit\n",
            n = name,
            b = end
        ));
        // corps
        for st in inner {
            match st {
                Stmt::Log(args) => emit_log(args, body),
                Stmt::For {
                    name,
                    start,
                    end,
                    body: inner2,
                } => emit_for(name, *start, *end, inner2, body, emit_log),
            }
        }
        // i++
        body.push_str(&format!(
            "    local.get ${n}\n    i32.const 1\n    i32.add\n    local.set ${n}\n",
            n = name
        ));
        // continue
        body.push_str("    br $loop\n    end\n    end\n");
    }

    for s in &prog.stmts {
        match s {
            Stmt::Log(args) => emit_log(args, &mut body),
            Stmt::For {
                name,
                start,
                end,
                body: inner,
            } => emit_for(name, *start, *end, inner, &mut body, &mut emit_log),
        }
    }

    // Locals: $pos + toutes les variables i32
    let mut locals = String::new();
    locals.push_str("    (local $pos i32)\n");
    for v in vars.keys() {
        locals.push_str(&format!("    (local ${} i32)\n", v));
    }

    // fonction util: i32 -> JSON (corrigée avec block/loop nommés)
    let i32_to_json = r#"
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
"#;

    // Assemble le module
    let mut wat = String::new();
    wat.push_str("(module\n");
    wat.push_str("  (import \"gaufre\" \"invoke\" (func $invoke (param i32 i32 i32 i32 i32 i32) (result i32)))\n");
    wat.push_str("  (memory (export \"memory\") 1)\n");
    wat.push_str(&data);
    wat.push_str(i32_to_json);
    wat.push_str("  (func (export \"main\")\n");
    wat.push_str(&locals);
    wat.push_str(&body);
    wat.push_str("  )\n");
    wat.push_str(")\n");

    wat
}
