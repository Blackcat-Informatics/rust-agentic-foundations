const fs=require('fs');
function replaceFn(s,name,body){const a=s.indexOf('fn '+name+'(');if(a<0)throw Error(name);let i=s.indexOf('{',a),depth=0,quoted=false,escaped=false;for(;i<s.length;i++){const c=s[i];if(quoted){if(escaped)escaped=false;else if(c==='\\')escaped=true;else if(c==='"')quoted=false;continue;}if(c==='"'){quoted=true;continue;}if(c==='{')depth++;if(c==='}'&&--depth===0)break;}return s.slice(0,a)+body+s.slice(i+1);}
function clean(s){return s.replace(/use serde_json[^\n]*\n/g,'').replaceAll('"json"','"turtle"').replace(/\barray,\s*/g,'').replace(/\bmember_names,\s*/g,'').replace(/\bstrings,\s*/g,'').replace(', strings};','};').replace('mod support;','mod support;\nuse support::rdf::Rdf;').replaceAll('assert_eq!(json.stdout, "[]\\n");','assert_eq!(Rdf::of(&json).result_count(), 0);');}
let p='crates/cargo-workplan/tests/history.rs',s=clean(fs.readFileSync(p,'utf8'));
s=replaceFn(s,'json_holds_the_entries_of_the_table_in_the_same_order',`fn rdf_preserves_full_history_commits_sources_paths_and_removed_bindings() {
 let repo=dated("history-rdf"); let text=repo.both(&["history","cap:alpha"]).expect(0);
 let run=repo.both(&["--format","turtle","history","cap:alpha"]).expect(0); let rdf=Rdf::of(&run);
 let entries=rdf.rows("HistoryEntry");
 let rows:Vec<&str>=text.stdout.lines().skip(2).take_while(|line|!line.is_empty()).collect();
 assert_eq!(entries.len(),rows.len());
 for (entry,row) in entries.iter().zip(&rows) {
   let commit=entry.required("commit"); assert_eq!(commit.len(),40);
   assert!(row.starts_with(&format!("| {} | ",&commit[..7])));
   assert!(row.contains(&format!("| {} |",entry.required("subject"))));
 }
 let source=entries[0].children("source"); assert_eq!(source.len(),1);
 assert_eq!(source[0].required("kind"),"symbol"); assert_eq!(source[0].required("binding"),"demo::real");
 assert_eq!(entries[0].children("path")[0].required("path"),"src/lib.rs");
 assert_eq!(entries[1].children("path")[0].required("path"),ALPHA);
 let removed=rdf.rows("RemovedBinding"); assert_eq!(removed.len(),1);
 assert_eq!(removed[0].required("binding"),"demo::gone"); assert_eq!(removed[0].required("removal"),"found");
 assert_eq!(removed[0].required("commit"),commit_of(&repo,"remove gone").as_str());
 assert_eq!(removed[0].required("identifier"),"gone");
 assert!(!removed[0].children("directory").is_empty());
 assert!(removed[0].text("lastResolved").is_some());
}`);fs.writeFileSync(p,s);
p='crates/cargo-workplan/tests/verify.rs';s=clean(fs.readFileSync(p,'utf8'));
s=replaceFn(s,'json_is_one_binding_object_for_each_binding',`fn rdf_preserves_real_binding_outcomes_proof_counts_and_quiet_filtering() {
 let repo=fixture("verify-rdf","[demo::tests::passes, demo::tests::fails]","[demo::real]");
 let run=repo.run(&["--format","turtle","verify","shape"]).expect(1);
 let cargo=support::run_in(repo.root(),support::Shape::Cargo,&["--format","turtle","verify","shape"]).expect(1);
 let outcomes=|run:&Run| { let rdf=Rdf::of(run); rdf.rows("Binding").iter().map(|r|(r.required("path").to_owned(),r.required("outcome").to_owned())).collect::<Vec<_>>() };
 assert_eq!(outcomes(&run),outcomes(&cargo));
 let rdf=Rdf::of(&run); let bindings=rdf.rows("Binding"); assert_eq!(bindings.len(),3);
 let (failed,passed,unresolved)=(bindings[0],bindings[1],bindings[2]);
 assert_eq!(failed.required("outcome"),"failed"); assert_eq!(failed.required("path"),"demo::tests::fails");
 assert_eq!(failed.required("list"),"tests"); assert_eq!(failed.required("capability"),"cap:fixture");
 assert_eq!((failed.number("listed"),failed.number("passed"),failed.number("failed"),failed.number("ignored")),(1,0,1,0));
 assert!(failed.required("reason").contains("exited with code 101"));
 assert_eq!(passed.required("outcome"),"passed"); assert!(passed.text("reason").is_none()); assert_eq!(passed.required("message"),PASS);
 assert_eq!(unresolved.required("outcome"),"unresolved"); assert_eq!(unresolved.required("list"),"examples");
 assert!(unresolved.required("reason").contains("no documentation test"));
 let text=repo.run(&["verify","shape"]);
 let messages=bindings.iter().map(|r|cut(r.required("message"))).collect::<Vec<_>>(); assert_eq!(steady(&text),messages);
 let quiet=repo.run(&["-q","--format","turtle","verify","shape"]).expect(1); let rdf=Rdf::of(&quiet); let kept=rdf.rows("Binding");
 assert_eq!(kept.len(),2); assert!(kept.iter().all(|r|r.required("outcome")!="passed"));
}`);fs.writeFileSync(p,s);
