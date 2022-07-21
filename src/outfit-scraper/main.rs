use anyhow::Result;
use convert_case::{Case, Casing};
use scraper::{Html, Selector};

const INDENT: &str = "    ";

fn get_outfits() -> Result<(Vec<String>, Vec<String>)> {
    let row_selector = Selector::parse("section.icon-set > div.row").unwrap();
    let col_selector = Selector::parse("div.col-12.col-sm-6").unwrap();
    let name_selector = Selector::parse("div.card-body > div.row p.text-center > small").unwrap();

    let mut heads = vec![];
    let mut tails = vec![];

    let resp = reqwest::blocking::get("https://play.battlesnake.com/references/customizations/")?;
    let doc = Html::parse_fragment(&resp.text()?);

    for row in doc.select(&row_selector) {
        let cols: Vec<_> = row.select(&col_selector).take(2).collect();

        for name in cols[0].select(&name_selector).map(|e| e.inner_html()) {
            tails.push(name)
        }

        for name in cols[1].select(&name_selector).map(|e| e.inner_html()) {
            heads.push(name)
        }
    }

    Ok((heads, tails))
}

fn print_enum(name: &str, members: &Vec<String>) {
    println!("#[derive(Serialize, Debug)]");
    println!("pub enum {} {{", name.to_case(Case::Pascal));
    for member in members {
        let identifier = member.to_case(Case::Pascal);
        println!("{}#[serde(rename = \"{}\")]", INDENT, member);
        println!("{}{},", INDENT, identifier);
    }
    println!("}}");
}

fn main() -> Result<()> {
    let (heads, tails) = get_outfits()?;

    print_enum("head", &heads);
    println!();
    print_enum("tail", &tails);

    Ok(())
}
