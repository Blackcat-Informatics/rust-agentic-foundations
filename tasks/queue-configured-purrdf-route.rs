// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A temporary exact native-provider route through the committed host Tree.
use raf_source::{CommittedGitTree, DiskTree, GitPackageRoots, LoadOptions, LockedCatalog,
    LockedEdge, LockedManifest, LockedPackageId, Namespace, RelPath, Workspace,
    CfgAtom,DeclaredCfg,DeclaredConfiguration,TargetKind};
use purrdf_lex::allocation::{Memory,Resident};
use std::error::Error;
const PIN:&str="5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43";
fn cfg_input(path:&std::path::Path)->Result<Vec<(String,Option<String>)>,Box<dyn Error>> {
    let raw=std::fs::read_to_string(path)?;
    if raw.len()>32768 {return Err("cfg input exceeds the explicit acquisition bound".into());}
    let mut resident=Resident;
    let mut memory=Memory::new(&mut resident);
    let mut source=String::new();
    for line in raw.lines() {
        let attr=memory.format(&format_args!("#![cfg({line})]\n"))?;
        memory.push_str(&mut source,&attr)?;
    }
    let parsed=raf_source::parse_source(&RelPath::new("declared-cfg.rs")?,&source,|tree| {
        let mut resident=Resident;
        let mut memory=Memory::new(&mut resident);
        let mut atoms=Vec::new();
        for attr in &tree.attrs {
            let meta=attr.parse_args::<syn::Meta>().map_err(|error|error.to_string())?;
            let (key,value)=match meta {
                syn::Meta::Path(path)=>(path,None),
                syn::Meta::NameValue(pair)=>{
                    let syn::Expr::Lit(lit)=pair.value else {return Err("cfg value is not a literal".to_owned());};
                    let syn::Lit::Str(value)=lit.lit else {return Err("cfg value is not text".to_owned());};
                    (pair.path,Some(value.value()))
                },
                _=>return Err("unexpected rustc cfg protocol".to_owned()),
            };
            let key=key.get_ident().ok_or_else(||"cfg key is not one identifier".to_owned())?.to_string();
            memory.push(&mut atoms,(key,value)).map_err(|error|error.to_string())?;
        }
        Ok::<_,String>(atoms)
    })??;
    Ok(parsed)
}
fn row(catalog:&LockedCatalog,name:&str)->Result<LockedPackageId,Box<dyn Error>> {
    let rows:Vec<_>=catalog.packages().filter(|row|row.name()==name&&row.version()=="3.0.1"&&row.source().is_some_and(|source|
        source.repository()=="https://github.com/Blackcat-Informatics/purrdf.git"&&source.revision()==Some(PIN))).collect();
    if rows.len()!=1 {return Err("no unique exact native row".into());}
    Ok(rows[0].id())
}
fn main()->Result<(),Box<dyn Error>> {
    let repository=std::env::args_os().nth(1).ok_or("supply existing pinned repository")?;
    let cfg_file=std::env::args_os().nth(2).ok_or("supply captured explicit rustc target cfg")?;
    let values=cfg_input(std::path::Path::new(&cfg_file))?;
    let mut resident=Resident;
    let mut memory=Memory::new(&mut resident);
    let mut atoms=Vec::new();
    for (name,value) in &values {
        memory.push(&mut atoms,match value {None=>CfgAtom::Name(name),Some(value)=>CfgAtom::Value(name,value)})?;
    }
    let declared=DeclaredCfg::complete("x86_64-unknown-linux-gnu",&atoms);
    let options=LoadOptions::default();
    let first=DiskTree::new(".")?;
    let catalog=LockedCatalog::load(&first,&RelPath::new("Cargo.lock")?,&options)?;
    let own:Vec<_>=catalog.packages().filter(|row|row.name()=="raf-source"&&row.source().is_none()).collect();
    if own.len()!=1 {return Err("no unique first-party row".into());}
    let lex=row(&catalog,"purrdf-lex")?;
    let hash=row(&catalog,"purrdf-hash")?;
    let tree=CommittedGitTree::open(&catalog,lex,&repository,&options)?;
    let own_edges=[LockedEdge::new("dependencies.purrdf-lex",None)];
    let local=[LockedManifest::new(own[0].id(),RelPath::new("crates/raf-source/Cargo.toml")?,&own_edges)
        .with_configuration(DeclaredConfiguration::new(declared,&[],TargetKind::Lib,"raf-source"))];
    let lex_edges=[LockedEdge::new("dependencies.purrdf-hash",None)];
    let native=[LockedManifest::new(lex,RelPath::new("crates/lex/Cargo.toml")?,&lex_edges)
        .with_configuration(DeclaredConfiguration::new(declared,&[],TargetKind::Lib,"purrdf_lex")),
        LockedManifest::new(hash,RelPath::new("crates/hash/Cargo.toml")?,&[])
        .with_configuration(DeclaredConfiguration::new(declared,&[],TargetKind::Lib,"purrdf_hash"))];
    let ws=Workspace::load_locked_sources(&catalog,&first,&local,&[GitPackageRoots::new(&tree,&native)],&options)?;
    let reading=ws.package_ids().find(|id|ws.package(*id).is_some_and(|package|package.name=="raf-source")).ok_or("reading package")?;
    for (written,canonical,file) in [("Memory","purrdf_lex::allocation::Memory","crates/lex/src/allocation.rs"),
        ("purrdf_lex::allocation::Shared","purrdf_lex::allocation::shared::Shared","crates/lex/src/allocation/shared.rs")] {
        let resolved=ws.resolve_active_from_package_id_in(reading,"raf_source::workspace",written,Namespace::Type)?;
        assert_eq!(resolved.path,canonical);
        assert!(!resolved.sites.is_empty());
        for site in resolved.sites {
            let item=ws.item(site).ok_or("definition")?;
            let target=ws.target(item.target).ok_or("target")?;
            let binding=ws.locked_binding(target.package_id).ok_or("checked binding")?;
            assert_eq!(item.file.as_str(),file);
            assert_eq!(item.file.authority().index(),1);
            assert_eq!(binding.row(),lex);
            assert_eq!(binding.source().and_then(|source|source.revision()),Some(PIN));
            let configured=binding.configuration().ok_or("definition is only structural")?;
            assert_eq!(configured.target().triple(),"x86_64-unknown-linux-gnu");
            assert_eq!(configured.target_kind(),TargetKind::Lib);
            assert_eq!(configured.features().len(),0);
            println!("{written} -> {} at {}:{}; actual package {} {}, commit {PIN}",item.path,item.file,item.name_line,binding.name(),binding.version());
        }
    }
    println!("joined usage: {:?}",ws.usage());
    Ok(())
}
