%start Prog
%%
Prog -> Result<Prog, ()>:
      Prog List '.'
      {
          let mut list = $1?;
          list.push($2?);
          Ok(list)
      }
    | List '.' { Ok(vec![$1?]) }
    ;

List -> Result<List, ()>:
      '{' Items '}' { $2 }
    ;

Items -> Result<List, ()>:
      Items ',' Item
      {
          let mut list = $1?;
          list.push($3?);
          Ok(list)
      }
    | Item { Ok(vec![$1?]) }
    ;

Item -> Result<Item, ()>:
      'INT'
      {
          let v = $1.map_err(|_| ())?;
          Ok(Item::Num(parse_int($lexer.span_str(v.span()))?))
      }
    | 'ATOM'
      {
          let v = $1.map_err(|_| ())?;
          Ok(Item::Atom($lexer.span_str(v.span()).to_string()))
      }
    | List { Ok(Item::List($1?)) }
    ;
%%
// Any functions here are in scope for all the grammar actions above.

pub type Prog = Vec<List>;
pub type List = Vec<Item>;
#[derive(Debug)]
pub enum Item {
    Num(u32),
    Atom(String),
    List(List),
}

impl Item {
    pub fn expect_num(&self) -> u32 {
        if let Item::Num(x) = self {
            *x
        } else {
            panic!("expected num, got {self:?}");
        }
    }
    pub fn expect_atom(&self) -> &str {
        if let Item::Atom(x) = self {
            x
        } else {
            panic!("expected atom, got {self:?}");
        }
    }
    pub fn expect_list(&self) -> &List {
        if let Item::List(x) = self {
            x
        } else {
            panic!("expected list, got {self:?}");
        }
    }
}

fn parse_int(s: &str) -> Result<u32, ()> {
    match s.parse::<u32>() {
        Ok(val) => Ok(val),
        Err(_) => {
            eprintln!("{s} cannot be represented as a u64");
            Err(())
        }
    }
}
