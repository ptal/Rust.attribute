// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use compile_error::*;
pub use syntax::parse::token::InternedString;
pub use syntax::ast::*;

use syntax::codemap::DUMMY_SP;
use std::rc::Rc;

pub struct AttributeValue<T>
{
  value: Option<T>,
  duplicate: DuplicateAttribute,
  span: Span
}

impl<T: Clone> AttributeValue<T>
{
  pub fn new(duplicate: DuplicateAttribute) -> AttributeValue<T>
  {
    AttributeValue {
      value: None,
      duplicate: duplicate,
      span: DUMMY_SP
    }
  }

  pub fn simple() -> AttributeValue<T>
  {
    AttributeValue::new(DuplicateAttribute::simple(Warn))
  }

  pub fn update(self, cx: &ExtCtxt, value: T, span: Span) -> AttributeValue<T>
  {
    let mut this = self;
    if this.value.is_some() {
      this.duplicate.issue(cx, this.span, span);
    } else {
      this.value = Some(value);
      this.span = span;
    }
    this
  }

  pub fn value_or(&self, default: T) -> T
  {
    match self.value {
      None => default,
      Some(ref value) => value.clone()
    }
  }

  pub fn span(&self) -> Span
  {
    self.span.clone()
  }
}

pub struct AttributeInfo
{
  pub name: InternedString,
  pub desc: &'static str,
  pub model: AttributeModel
}

impl AttributeInfo
{
  pub fn new(name: &'static str, desc: &'static str, 
    model: AttributeModel) -> AttributeInfo
  {
    AttributeInfo {
      name: InternedString::new(name),
      desc: desc,
      model: model
    }
  }

  pub fn simple(name: &'static str, desc: &'static str) -> AttributeInfo
  {
    AttributeInfo::new(name, desc, NoProperty(AttributeValue::simple()))
  }

  pub fn update(self, model: AttributeModel) -> AttributeInfo
  {
    let mut this = self;
    this.model = model;
    this
  }

  pub fn plain_value<'a>(&'a self) -> &'a AttributeValue<bool>
  {
    match self.model {
      NoProperty(ref v) => v,
      _ => fail!("No plain value for the current attribute.")
    }
  }

  pub fn sub_model<'a>(&'a self) -> &'a AttributeDict
  {
    match self.model {
      SubAttribute(ref dict) => dict,
      _ => fail!("No sub value for the attribute.")
    }
  }
}

pub enum AttributeModel
{
  NoProperty(AttributeValue<bool>),
  KeyValue(AttributeLitModel),
  SubAttribute(AttributeDict)
}

pub enum AttributeLitModel
{
  MLitStr(AttributeValue<(InternedString, StrStyle)>),
  MLitBinary(AttributeValue<Rc<Vec<u8>>>),
  MLitByte(AttributeValue<u8>),
  MLitChar(AttributeValue<char>),
  MLitInt(AttributeValue<(u64, LitIntType)>),
  MLitFloat(AttributeValue<(InternedString, FloatTy)>),
  MLitFloatUnsuffixed(AttributeValue<InternedString>),
  MLitNil(AttributeValue<()>),
  MLitBool(AttributeValue<bool>)
}

pub struct AttributeDict
{
  dict: Vec<AttributeInfo>
}

impl AttributeDict
{
  pub fn new(dict: Vec<AttributeInfo>) -> AttributeDict
  {
    AttributeDict {
      dict: dict
    }
  }

  pub fn move_map(self, f: |AttributeInfo| -> AttributeInfo) -> AttributeDict
  {
    let mut this = self;
    this.dict = this.dict.move_iter().map(f).collect();
    this
  }

  pub fn push(&mut self, attr: AttributeInfo)
  {
    self.dict.push(attr);
  }

  pub fn push_all(&mut self, attrs: Vec<AttributeInfo>)
  {
    self.dict.push_all_move(attrs);
  }

  pub fn get<'a>(&'a self, name: &'static str) -> &'a AttributeInfo
  {
    let interned = InternedString::new(name);
    for info in self.dict.iter() {
      if info.name == interned {
        return info;
      }
    }
    fail!("Try to get an attribute that doesn't exist.")
  }

  pub fn plain_value<'a>(&'a self, name: &'static str) -> &'a AttributeValue<bool>
  {
    self.get(name).plain_value()
  }

  pub fn sub_model<'a>(&'a self, name: &'static str) -> &'a AttributeDict
  {
    self.get(name).sub_model()
  }

  pub fn plain_value_or(&self, name: &'static str, def: bool) -> bool
  {
    self.plain_value(name).value_or(def)
  }

  // fn attribute_doc(&self)
  // {
  //   let mut doc = format!("Attribute `#[{}(<attribute list>)]`: {}\n",
  //       self.root_name.get(), self.root_desc);
  //   for info in self.infos.iter() {
  //     doc.add(format!("  * `#[{}({})]`: {}\n",
  //       self.root_name.get(), info.name, info.desc));
  //   }
  //   self.cx.parse_sess.span_diagnostic.handler.note(doc.as_slice());
  // }
}


impl AttributeLitModel
{
  pub fn to_lit_printer(&self) -> LitTypePrinter
  {
    match *self {
      MLitStr(_) => PLitStr,
      MLitBinary(_) => PLitBinary,
      MLitByte(_) => PLitByte,
      MLitChar(_) => PLitChar,
      MLitInt(_) => PLitInt,
      MLitFloat(_) => PLitFloat,
      MLitFloatUnsuffixed(_) => PLitFloatUnsuffixed,
      MLitNil(_) => PLitNil,
      MLitBool(_) => PLitBool
    }
  }
}

pub fn lit_to_lit_printer(lit: &Lit_) -> LitTypePrinter
{
  match *lit {
    LitStr(_, _) => PLitStr,
    LitBinary(_) => PLitBinary,
    LitByte(_) => PLitByte,
    LitChar(_) => PLitChar,
    LitInt(_, _) => PLitInt,
    LitFloat(_, _) => PLitFloat,
    LitFloatUnsuffixed(_) => PLitFloatUnsuffixed,
    LitNil => PLitNil,
    LitBool(_) => PLitBool
  }
}

pub enum LitTypePrinter
{
  PLitStr,
  PLitBinary,
  PLitByte,
  PLitChar,
  PLitInt,
  PLitFloat,
  PLitFloatUnsuffixed,
  PLitNil,
  PLitBool
}

impl LitTypePrinter
{
  pub fn type_to_str(&self) -> &'static str
  {
    match *self {
      PLitStr => "string",
      PLitBinary => "binary",
      PLitByte => "byte",
      PLitChar => "char",
      PLitInt => "int",
      PLitFloat => "float",
      PLitFloatUnsuffixed => "unsuffixed float",
      PLitNil => "nil",
      PLitBool => "boolean"
    }
  }

  pub fn type_example_to_str(&self) -> &'static str
  {
    match *self {
      PLitStr => "\"hello world\"",
      PLitBinary => "0b01010101",
      PLitByte => "b'9'",
      PLitChar => "'a'",
      PLitInt => "38",
      PLitFloat => "0.01f32",
      PLitFloatUnsuffixed => "0.1",
      PLitNil => "()",
      PLitBool => "true"
    }
  }
}
