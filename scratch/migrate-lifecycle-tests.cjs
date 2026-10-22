const fs=require('fs');
function replaceFn(s,name,body){const a=s.indexOf('fn '+name+'(');if(a<0)throw Error(name);let i=s.indexOf('{',a),depth=0,quoted=false,escaped=false;for(;i<s.length;i++){const c=s[i];if(quoted){if(escaped)escaped=false;else if(c==='\\')escaped=true;else if(c==='"')quoted=false;continue;}if(c==='"'){quoted=true;continue;}if(c==='{')depth++;if(c==='}'&&--depth===0)break;}return s.slice(0,a)+body+s.slice(i+1);}
function imports(s){return s.replace(/use serde_json[^\n]*\n/g,'').replaceAll('"json"','"turtle"').replace(/\barray,\s*/g,'').replace(/\bmember_names,\s*/g,'').replace(/\bstrings,\s*/g,'').replace('mod support;','mod support;\nuse support::rdf::Rdf;');}
let p='crates/cargo-workplan/tests/new.rs',s=imports(fs.readFileSync(p,'utf8'));
s=replaceFn(s,'json_is_one_new_change_object_with_its_fields',`fn rdf_preserves_created_change_fields_and_ordered_guidance() {
 let repo=Repo::synced("new-rdf",false);
 let args=with(&["--affects","cap:beta#usage"]);
 let mut words=vec!["--format","turtle"]; words.extend(args.iter().map(String::as_str));
 let run=repo.run(&words).expect(0); let rdf=Rdf::of(&run); let rows=rdf.rows("NewChange");
 assert_eq!(rows.len(),1); let change=rows[0];
 assert_eq!(change.required("slug"),"fresh-change"); assert_eq!(change.required("path"),RECORD);
 assert_eq!(change.children("capability").iter().map(|r|r.required("id")).collect::<Vec<_>>(),["cap:alpha","cap:beta"]);
 assert_eq!(change.children("affects")[0].required("subject"),"cap:beta#usage");
 assert_eq!(change.required("template"),"> [!PROPOSED change:fresh-change]\\n> State the proposed text here.\\n");
 let guidance=change.children("guidance"); assert_eq!(guidance.len(),2);
 assert_eq!((guidance[0].required("advice"),guidance[0].required("capability"),guidance[0].required("state")),("set-implementing","cap:alpha","delivered"));
 assert_eq!((guidance[1].required("advice"),guidance[1].required("capability"),guidance[1].required("state")),("keep","cap:beta","implementing"));
 assert!(!change.boolean("syncNeeded"));
}`);fs.writeFileSync(p,s);
p='crates/cargo-workplan/tests/impact.rs';s=imports(fs.readFileSync(p,'utf8'));
s=replaceFn(s,'json_is_one_impact_object_with_its_lists',`fn rdf_preserves_impact_bindings_dependencies_links_and_list_order() {
 let repo=Repo::synced("impact-rdf",true);
 let run=repo.both(&["--format","turtle","change","impact","grow"]).expect(0);
 let rdf=Rdf::of(&run); let rows=rdf.rows("Impact"); assert_eq!(rows.len(),1); let impact=rows[0];
 assert_eq!(impact.required("change"),"grow");
 assert_eq!(impact.children("capability").iter().map(|r|r.required("id")).collect::<Vec<_>>(),["cap:beta","cap:delta","cap:gamma"]);
 assert_eq!(impact.children("section").iter().map(|r|r.required("subject")).collect::<Vec<_>>(),["cap:beta#contract","cap:delta#contract","cap:gamma#contract"]);
 let bindings=impact.children("binding"); assert_eq!(bindings.len(),2);
 assert_eq!(bindings[0].required("path"),"demo::missing"); assert!(!bindings[0].boolean("resolves")); assert!(bindings[0].text("reason").is_some());
 assert_eq!(bindings[0].required("list"),"code"); assert_eq!(bindings[0].required("capability"),"cap:beta");
 assert_eq!(bindings[1].required("path"),"demo::real"); assert!(bindings[1].boolean("resolves")); assert!(bindings[1].text("reason").is_none());
 assert!(impact.children("dependant").is_empty()); assert!(impact.children("link").is_empty());
 let repo=Repo::synced("impact-rdf-reach",false);
 repo.put("work/changes/watch.md",&change("watch","[cap:alpha]",&["cap:alpha#contract"]));
 repo.edit("docs/capabilities/gamma.md","Text for Usage of gamma.","Text for Usage of gamma, see [it](alpha.md#contract)."); repo.sync();
 let run=repo.run(&["--format","turtle","change","impact","watch"]).expect(0); let rdf=Rdf::of(&run); let impact=rdf.rows("Impact")[0];
 let dependants=impact.children("dependant"); assert_eq!(dependants.len(),2);
 assert_eq!(dependants.iter().map(|d|(d.required("capability"),d.required("dependsOn"))).collect::<Vec<_>>(),[("cap:beta","cap:alpha"),("cap:gamma","cap:alpha")]);
 let links=impact.children("link"); assert_eq!(links.len(),1);
 assert_eq!((links[0].required("from"),links[0].required("target")),("cap:gamma#usage","cap:alpha#contract"));
}`);fs.writeFileSync(p,s);
p='crates/cargo-workplan/tests/reconcile.rs';s=imports(fs.readFileSync(p,'utf8')).replace('assert_eq!(json.stdout, "[]\\n");','assert_eq!(Rdf::of(&json).result_count(),0);');
s=replaceFn(s,'json_is_one_obligation_object_for_each_obligation',`fn rdf_preserves_each_obligation_and_optional_location_in_library_order() {
 let repo=checkout("reconcile-rdf"); let run=repo.both(&["--format","turtle","change","reconcile","grow"]).expect(1);
 let rdf=Rdf::of(&run); let obligations=rdf.rows("Obligation"); assert_eq!(obligations.len(),6);
 let text:Run=repo.run(&["change","reconcile","grow"]);
 for (row,line) in obligations.iter().zip(text.lines()) {
   let fields=[row.required("subject"),row.required("section"),row.required("kind")];
   assert!(line.starts_with(&fields.join("  "))); assert!(line.ends_with(row.required("evidence")));
 }
 let proposal=obligations.iter().find(|r|r.required("kind")=="unresolved-proposal").unwrap();
 assert_eq!(proposal.required("subject"),"cap:delta"); assert_eq!(proposal.required("section"),"contract");
 assert_eq!(proposal.required("path"),"docs/capabilities/delta.md"); assert_eq!(proposal.number("line"),26);
 assert_eq!(proposal.required("evidence"),"the block is replaced by plain contract text, or removed because the claim was dropped");
 let untouched=obligations.iter().find(|r|r.required("kind")=="section-untouched").unwrap(); assert!(untouched.text("line").is_none());
 let quiet=repo.run(&["-q","--format","turtle","change","reconcile","grow"]).expect(1); assert_eq!(quiet.stdout,run.stdout);
}`);fs.writeFileSync(p,s);
