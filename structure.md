.
├── LICENSE
├── Makefile
├── README.md
├── RelayDB-Logo.png
├── RelayDB_v2_Final_Project_Specification.md
├── data
│   ├── actors.json
│   ├── directors.json
│   └── movies.json
├── horizontal_test.png
├── relay-compiler
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── builds
│   ├── src
│   │   ├── bin
│   │   │   ├── compiler.rs
│   │   │   ├── reader.rs
│   │   │   ├── relay.rs
│   │   │   ├── v2_compiler.rs
│   │   │   └── verify.rs
│   │   ├── lib.rs
│   │   └── v2_protocol.rs
│   └── target
│       ├── debug
│       │   ├── build
│       │   │   ├── num-traits-13e867a0f1d16f26
│       │   │   │   ├── invoked.timestamp
│       │   │   │   ├── out
│       │   │   │   ├── output
│       │   │   │   ├── root-output
│       │   │   │   └── stderr
│       │   │   ├── num-traits-470d939f1bfde4cd
│       │   │   │   ├── build-script-build
│       │   │   │   ├── build_script_build-470d939f1bfde4cd
│       │   │   │   └── build_script_build-470d939f1bfde4cd.d
│       │   │   ├── proc-macro2-9bfd32320f1a573f
│       │   │   │   ├── build-script-build
│       │   │   │   ├── build_script_build-9bfd32320f1a573f
│       │   │   │   └── build_script_build-9bfd32320f1a573f.d
│       │   │   ├── proc-macro2-d1d6416a93bdc668
│       │   │   │   ├── invoked.timestamp
│       │   │   │   ├── out
│       │   │   │   ├── output
│       │   │   │   ├── root-output
│       │   │   │   └── stderr
│       │   │   ├── quote-672b7286808da96b
│       │   │   │   ├── invoked.timestamp
│       │   │   │   ├── out
│       │   │   │   ├── output
│       │   │   │   ├── root-output
│       │   │   │   └── stderr
│       │   │   ├── quote-791f82315cf5282a
│       │   │   │   ├── build-script-build
│       │   │   │   ├── build_script_build-791f82315cf5282a
│       │   │   │   └── build_script_build-791f82315cf5282a.d
│       │   │   ├── serde-0e8e8a817c3d4882
│       │   │   │   ├── invoked.timestamp
│       │   │   │   ├── out
│       │   │   │   │   └── private.rs
│       │   │   │   ├── output
│       │   │   │   ├── root-output
│       │   │   │   └── stderr
│       │   │   ├── serde-ddee6aa307d310dd
│       │   │   │   ├── build-script-build
│       │   │   │   ├── build_script_build-ddee6aa307d310dd
│       │   │   │   └── build_script_build-ddee6aa307d310dd.d
│       │   │   ├── serde_core-558dc67b726cd3b0
│       │   │   │   ├── build-script-build
│       │   │   │   ├── build_script_build-558dc67b726cd3b0
│       │   │   │   └── build_script_build-558dc67b726cd3b0.d
│       │   │   ├── serde_core-a18c5bc705536fe8
│       │   │   │   ├── invoked.timestamp
│       │   │   │   ├── out
│       │   │   │   │   └── private.rs
│       │   │   │   ├── output
│       │   │   │   ├── root-output
│       │   │   │   └── stderr
│       │   │   ├── serde_json-c2a4b8aaa55ba5c0
│       │   │   │   ├── build-script-build
│       │   │   │   ├── build_script_build-c2a4b8aaa55ba5c0
│       │   │   │   └── build_script_build-c2a4b8aaa55ba5c0.d
│       │   │   ├── serde_json-f4fc0051018ba0b0
│       │   │   │   ├── invoked.timestamp
│       │   │   │   ├── out
│       │   │   │   ├── output
│       │   │   │   ├── root-output
│       │   │   │   └── stderr
│       │   │   ├── zmij-0d15141a4c2470b9
│       │   │   │   ├── invoked.timestamp
│       │   │   │   ├── out
│       │   │   │   ├── output
│       │   │   │   ├── root-output
│       │   │   │   └── stderr
│       │   │   └── zmij-e1dbd4c4b53bed3e
│       │   │       ├── build-script-build
│       │   │       ├── build_script_build-e1dbd4c4b53bed3e
│       │   │       └── build_script_build-e1dbd4c4b53bed3e.d
│       │   ├── deps
│       │   │   ├── anstream-44b03e72ace4bed3.d
│       │   │   ├── anstyle-930fa47b2e73eb6e.d
│       │   │   ├── anstyle_parse-761e9969cd541738.d
│       │   │   ├── anstyle_query-a06dc0280f5639a8.d
│       │   │   ├── autocfg-12f89cc5520a8a2d.d
│       │   │   ├── chrono-eb91943bf59523f8.d
│       │   │   ├── clap-756da6ac180ba84c.d
│       │   │   ├── clap_builder-a4f550640ff843a5.d
│       │   │   ├── clap_derive-5214be8d11ce0b1b.d
│       │   │   ├── clap_lex-1e59fc455e3ba4fd.d
│       │   │   ├── colorchoice-851a1169fabacc05.d
│       │   │   ├── compiler-77252a3a24f85733.d
│       │   │   ├── compiler-b6321a68c204ad5b.d
│       │   │   ├── core_foundation_sys-4693dae6dbaa0ffe.d
│       │   │   ├── heck-5b561473ea84a355.d
│       │   │   ├── iana_time_zone-decec4afa484a4ab.d
│       │   │   ├── is_terminal_polyfill-bcb34d47b71917f1.d
│       │   │   ├── itoa-4af1759b6c4aff3d.d
│       │   │   ├── libanstream-44b03e72ace4bed3.rmeta
│       │   │   ├── libanstyle-930fa47b2e73eb6e.rmeta
│       │   │   ├── libanstyle_parse-761e9969cd541738.rmeta
│       │   │   ├── libanstyle_query-a06dc0280f5639a8.rmeta
│       │   │   ├── libautocfg-12f89cc5520a8a2d.rlib
│       │   │   ├── libautocfg-12f89cc5520a8a2d.rmeta
│       │   │   ├── libchrono-eb91943bf59523f8.rmeta
│       │   │   ├── libclap-756da6ac180ba84c.rmeta
│       │   │   ├── libclap_builder-a4f550640ff843a5.rmeta
│       │   │   ├── libclap_derive-5214be8d11ce0b1b.dylib
│       │   │   ├── libclap_lex-1e59fc455e3ba4fd.rmeta
│       │   │   ├── libcolorchoice-851a1169fabacc05.rmeta
│       │   │   ├── libcompiler-77252a3a24f85733.rmeta
│       │   │   ├── libcompiler-b6321a68c204ad5b.rmeta
│       │   │   ├── libcore_foundation_sys-4693dae6dbaa0ffe.rmeta
│       │   │   ├── libheck-5b561473ea84a355.rlib
│       │   │   ├── libheck-5b561473ea84a355.rmeta
│       │   │   ├── libiana_time_zone-decec4afa484a4ab.rmeta
│       │   │   ├── libis_terminal_polyfill-bcb34d47b71917f1.rmeta
│       │   │   ├── libitoa-4af1759b6c4aff3d.rmeta
│       │   │   ├── libmemchr-fb37fa198ad042a8.rmeta
│       │   │   ├── libnum_traits-c0d4ef04e02e972d.rmeta
│       │   │   ├── libproc_macro2-bc8e63d3fdc6c233.rlib
│       │   │   ├── libproc_macro2-bc8e63d3fdc6c233.rmeta
│       │   │   ├── libquote-853fa6697aa57409.rlib
│       │   │   ├── libquote-853fa6697aa57409.rmeta
│       │   │   ├── libreader-eb093c955ce56cf2.rmeta
│       │   │   ├── libreader-ff6577a43a85be91.rmeta
│       │   │   ├── librelay-141207f0f07a79ae.rmeta
│       │   │   ├── librelay-612c99bb35f77df4.rmeta
│       │   │   ├── librelay_compiler-50cfcb3e3aec3dd9.rmeta
│       │   │   ├── librelay_compiler-9bae6841d52b438a.rmeta
│       │   │   ├── libserde-2ce0b4e0e0a5bb54.rmeta
│       │   │   ├── libserde_core-c8a5382a4629af28.rmeta
│       │   │   ├── libserde_derive-040d58a5079851f8.dylib
│       │   │   ├── libserde_json-8140ffcc2cbec46f.rmeta
│       │   │   ├── libstrsim-1b13e726b727ef4d.rmeta
│       │   │   ├── libsyn-261fc204d261455c.rlib
│       │   │   ├── libsyn-261fc204d261455c.rmeta
│       │   │   ├── libunicode_ident-a19fae25a22ef471.rlib
│       │   │   ├── libunicode_ident-a19fae25a22ef471.rmeta
│       │   │   ├── libutf8parse-f544a444e482889a.rmeta
│       │   │   ├── libv2_compiler-3c2bd46c42a38f7d.rmeta
│       │   │   ├── libverify-5dc7afdd2a6e3cd1.rmeta
│       │   │   ├── libverify-7eb3810d0a8b0b7f.rmeta
│       │   │   ├── libzmij-0e6add05bada4fa2.rmeta
│       │   │   ├── memchr-fb37fa198ad042a8.d
│       │   │   ├── num_traits-c0d4ef04e02e972d.d
│       │   │   ├── proc_macro2-bc8e63d3fdc6c233.d
│       │   │   ├── quote-853fa6697aa57409.d
│       │   │   ├── reader-eb093c955ce56cf2.d
│       │   │   ├── reader-ff6577a43a85be91.d
│       │   │   ├── relay-141207f0f07a79ae.d
│       │   │   ├── relay-612c99bb35f77df4.d
│       │   │   ├── relay_compiler-50cfcb3e3aec3dd9.d
│       │   │   ├── relay_compiler-9bae6841d52b438a.d
│       │   │   ├── serde-2ce0b4e0e0a5bb54.d
│       │   │   ├── serde_core-c8a5382a4629af28.d
│       │   │   ├── serde_derive-040d58a5079851f8.d
│       │   │   ├── serde_json-8140ffcc2cbec46f.d
│       │   │   ├── strsim-1b13e726b727ef4d.d
│       │   │   ├── syn-261fc204d261455c.d
│       │   │   ├── unicode_ident-a19fae25a22ef471.d
│       │   │   ├── utf8parse-f544a444e482889a.d
│       │   │   ├── v2_compiler-3c2bd46c42a38f7d.d
│       │   │   ├── v2_compiler-e2a5b6551c118af8.d
│       │   │   ├── verify-5dc7afdd2a6e3cd1.d
│       │   │   ├── verify-7eb3810d0a8b0b7f.d
│       │   │   └── zmij-0e6add05bada4fa2.d
│       │   ├── examples
│       │   └── incremental
│       │       ├── compiler-17aezynznoiuh
│       │       │   ├── s-hi2m7glmhl-00ish4q-aqx25wqey44pb3j05noy9d7th
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7glmhl-00ish4q.lock
│       │       ├── compiler-1mp97bvjxxcgz
│       │       │   ├── s-hi2m7glqt7-0auc025-2yarulrjzpmfau8pd0vhwl4m9
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7glqt7-0auc025.lock
│       │       ├── reader-0dtvls8st07oz
│       │       │   ├── s-hi2m7glo9r-1uygzy6-alyr1r0x6s7d6ytju9fhz83c8
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7glo9r-1uygzy6.lock
│       │       ├── reader-31rv03pk76p4d
│       │       │   ├── s-hi2m7glpmg-0wbgckg-1ppzszeu3n2yy2iwybnhy74xp
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7glpmg-0wbgckg.lock
│       │       ├── relay-0kh6ns4hphmn8
│       │       │   ├── s-hi2m7gll8r-020m5f6-8kd6ab1tndk262v0g8hoe735d
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7gll8r-020m5f6.lock
│       │       ├── relay-364s5y0vla2qp
│       │       │   ├── s-hi2m7gllca-08esmfp-8jgnkwq9dkdgnw7sqeduefaf4
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7gllca-08esmfp.lock
│       │       ├── relay_compiler-0bo0m8oo2e5hs
│       │       │   ├── s-hi2m7gkoq3-0wleup9-4pb9btl8600yllactk1fe6frn
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── metadata.rmeta
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7gkoq3-0wleup9.lock
│       │       ├── relay_compiler-0uebpkrgs76i9
│       │       │   ├── s-hi2m7gkopq-16d8jfh-2xhcg3e834bq66cs5ziuvi7hx
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7gkopq-16d8jfh.lock
│       │       ├── v2_compiler-2o7lojg2r8grx
│       │       │   ├── s-hi2m7glma1-1gkzg4t-working
│       │       │   │   └── dep-graph.part.bin
│       │       │   └── s-hi2m7glma1-1gkzg4t.lock
│       │       ├── v2_compiler-3ejfpoblh0k9a
│       │       │   ├── s-hi2m7glm5t-18ivklb-6h7wqzebvthoe5hj14jt3hp1v
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7glm5t-18ivklb.lock
│       │       ├── verify-00jyax31kif29
│       │       │   ├── s-hi2m7gll7n-0v9ddve-eln11r20xq5r5rpmz3ko5dwtc
│       │       │   │   ├── dep-graph.bin
│       │       │   │   ├── query-cache.bin
│       │       │   │   └── work-products.bin
│       │       │   └── s-hi2m7gll7n-0v9ddve.lock
│       │       └── verify-1yf9kdb5obnyq
│       │           ├── s-hi2m7glqia-05g9fqi-0njqazhxwkj9eo8b9aw9ltc4u
│       │           │   ├── dep-graph.bin
│       │           │   ├── query-cache.bin
│       │           │   └── work-products.bin
│       │           └── s-hi2m7glqia-05g9fqi.lock
│       └── flycheck0
│           ├── stderr
│           └── stdout
└── structure.md

58 directories, 206 files
