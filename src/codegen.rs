use crate::parser::Program;

/// Échappe une chaîne pour un littéral WAT: 
/// - garde ASCII imprimable 0x20..0x7E sauf " et \
/// - remplace " -> \" ; \ -> \\ ; \n,\r,\t -> échappements ; le reste en \hh (hex)
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

/// Génère le WAT pour Program { logs }
pub fn generate_wat(prog: &Program) -> String {
    // Offsets mémoire:
    // - 0.. : "console.log" (len=11)
    // - ensuite: pour chaque log, un segment data avec l'array JSON ["..."]
    // - buffer retour pour invoke: 4096 (cap 1024)
    let name_console_log = "console.log";
    let name_len = name_console_log.len(); // 11

    // Commence à 16 pour aligner un peu (lisibilité)
    let mut next_off: usize = 16;

    // Prépare les segments (offset, contenu) des args JSON
    struct ArgSeg { off: usize, len: usize }
    let mut arg_segs: Vec<ArgSeg> = Vec::with_capacity(prog.logs.len());

    // Construction des data segments
    let mut data_segments = String::new();

    // Segment pour "console.log"
    data_segments.push_str(&format!(
        "  (data (i32.const 0) \"{}\")\n",
        wat_escape(name_console_log),
    ));

    // Segments pour chaque ["..."]
    for s in &prog.logs {
        // JSON de l'argument: ["..."] avec échappements JSON corrects
        let arg_json = format!("[{}]", serde_json::to_string(s).unwrap());
        let arg_len = arg_json.as_bytes().len();
        data_segments.push_str(&format!(
            "  (data (i32.const {off}) \"{lit}\")\n",
            off = next_off,
            lit = wat_escape(&arg_json),
        ));
        arg_segs.push(ArgSeg { off: next_off, len: arg_len });

        // avance (petit alignement 16 pour faciliter les évolutions)
        next_off += arg_len;
        if next_off % 16 != 0 { next_off += 16 - (next_off % 16); }
    }

    // Corps de la fonction main: un call $invoke par log(...)
    let mut body = String::new();
    for ArgSeg { off, len } in arg_segs {
        body.push_str(&format!(
            concat!(
                "    i32.const 0      ;; name ptr: \"console.log\"\n",
                "    i32.const {name_len}\n",
                "    i32.const {args_off}  ;; args ptr: [\"...\"] (JSON)\n",
                "    i32.const {args_len}\n",
                "    i32.const 4096   ;; ret buf ptr\n",
                "    i32.const 1024   ;; ret cap\n",
                "    call $invoke\n",
                "    drop             ;; ignore le retour (console.log -> null)\n",
            ),
            name_len = name_len,
            args_off = off,
            args_len = len,
        ));
    }

    // Assemble le module
    let mut wat = String::new();
    wat.push_str("(module\n");
    wat.push_str("  (import \"gaufre\" \"invoke\" (func $invoke (param i32 i32 i32 i32 i32 i32) (result i32)))\n");
    wat.push_str("  (memory (export \"memory\") 1)\n");
    wat.push_str(&data_segments);
    wat.push_str("  (func (export \"main\")\n");
    if body.is_empty() {
        // Aucun log: ne rien faire (module valide)
        wat.push_str("    ;; aucun log\n");
    } else {
        wat.push_str(&body);
    }
    wat.push_str("  )\n");
    wat.push_str(")\n");
    wat
}
