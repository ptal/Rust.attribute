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

#[deriving(Clone)]
pub struct AttributeValue<T>
{
  pub value: Option<T>,
  pub span: Span,
  duplicate: DuplicateAttribute
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

  pub fn has_value(&self) -> bool
  {
    self.value.is_some()
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

#[deriving(Clone)]
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
    AttributeInfo::new(name, desc, UnitValue(AttributeValue::simple()))
  }

  pub fn update(self, model: AttributeModel) -> AttributeInfo
  {
    let mut this = self;
    this.model = model;
    this
  }

  pub fn plain_value<'a>(&'a self) -> &'a AttributeValue<()>
  {
    match self.model {
      UnitValue(ref v) => v,
      _ => fail!("No plain value for the current attribute.")
    }
  }

  pub fn sub_model<'a>(&'a self) -> &'a AttributeArray
  {
    match self.model {
      SubAttribute(ref array) => array,
      _ => fail!("No sub value for the current attribute.")
    }
  }

  pub fn key_value<'a>(&'a self) -> &'a AttributeLitModel
  {
    match self.model {
      KeyValue(ref lit) => lit,
      _ => fail!("No key value for the current attribute.")
    }
  }
}

#[deriving(Clone)]
pub enum AttributeModel
{
  UnitValue(AttributeValue<()>),
  KeyValue(AttributeLitModel),
  SubAttribute(AttributeArray)
}

#[deriving(Clone)]
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

pub type AttributeArray = Vec<AttributeInfo>;

pub mod access
{
  pub use super::*;
  pub fn by_name<'a>(array: &'a AttributeArray, name: &'static str) -> &'a AttributeInfo
  {
    let interned = InternedString::new(name);
    for info in array.iter() {
      if info.name == interned {
        return info;
      }
    }
    fail!("Try to get an attribute that doesn't exist.")
  }

  pub fn plain_value<'a>(array: &'a AttributeArray, name: &'static str) -> &'a AttributeValue<()>
  {
    by_name(array, name).plain_value()
  }

  pub fn sub_model<'a>(array: &'a AttributeArray, name: &'static str) -> &'a AttributeArray
  {
    by_name(array, name).sub_model()
  }

  pub fn lit_str<'a>(array: &'a AttributeArray, name: &'static str) -> &'a AttributeValue<(InternedString, StrStyle)>
  {
    match by_name(array, name).key_value() {
      &MLitStr(ref val) => val,
      _ => fail!("No string literal available for this attribute.")
    }
  }

  pub fn plain_value_or(array: &AttributeArray, name: &'static str, def: bool) -> bool
  {
    if plain_value(array, name).has_value() {
      true
    } else {
      def
    }
  }
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

pub struct AttributeMerger<'a>
{
  cx: &'a ExtCtxt<'a>,
  duplicate: DuplicateAttribute
}

impl<'a> AttributeMerger<'a>
{
  pub fn new(cx: &'a ExtCtxt, duplicate: DuplicateAttribute) -> AttributeMerger<'a>
  {
    AttributeMerger {
      cx: cx,
      duplicate: duplicate
    }
  }

  pub fn merge(&self, info: AttributeInfo, info2: AttributeInfo) -> AttributeInfo
  {
    assert!(info.name == info2.name);
    let mut info = info;
    info.model = match (info.model, info2.model) {
      (UnitValue(val), UnitValue(val2)) => UnitValue(self.merge_value(val, val2)),
      (KeyValue(lit), KeyValue(lit2)) => KeyValue(self.merge_lit(lit, lit2)),
      (SubAttribute(sub), SubAttribute(sub2)) => SubAttribute(self.merge_sub_attr(sub, sub2)),
      _ => fail!("Mismatch between attribute models during merging.")
    };
    info
  }

  fn merge_value<T>(&self, val: AttributeValue<T>, val2: AttributeValue<T>) -> AttributeValue<T>
  {
    match (&val.value, &val2.value) {
      (&None, _) => val2,
      (_, &None) => val,
      (&Some(_), &Some(_)) => {
        self.duplicate.issue(self.cx, val.span, val2.span);
        val
      }
    }
  }

  fn merge_lit(&self, lit: AttributeLitModel, lit2: AttributeLitModel) -> AttributeLitModel
  {
    match (lit, lit2) {
      (MLitStr(val), MLitStr(val2)) => MLitStr(self.merge_value(val, val2)),
      (MLitBinary(val), MLitBinary(val2)) => MLitBinary(self.merge_value(val, val2)),
      (MLitByte(val), MLitByte(val2)) => MLitByte(self.merge_value(val, val2)),
      (MLitChar(val), MLitChar(val2)) => MLitChar(self.merge_value(val, val2)),
      (MLitInt(val), MLitInt(val2)) => MLitInt(self.merge_value(val, val2)),
      (MLitFloat(val), MLitFloat(val2)) => MLitFloat(self.merge_value(val, val2)),
      (MLitFloatUnsuffixed(val), MLitFloatUnsuffixed(val2)) => MLitFloatUnsuffixed(self.merge_value(val, val2)),
      (MLitNil(val), MLitNil(val2)) => MLitNil(self.merge_value(val, val2)),
      (MLitBool(val), MLitBool(val2)) => MLitBool(self.merge_value(val, val2)),
      _ => fail!("Mismatch between attribute models during merging.")
    }
  }

  fn merge_sub_attr(&self, sub1: AttributeArray, sub2: AttributeArray) -> AttributeArray
  {
    assert!(sub1.len() == sub2.len());
    sub1.move_iter()
      .zip(sub2.move_iter())
      .map(|(info, info2)| self.merge(info, info2))
      .collect()
  }
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
