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

use model::*;
use syntax::ptr::P;

use model::AttributeModel::*;
use model::AttributeLitModel::*;

pub fn check_all(cx: &ExtCtxt, model: AttributeArray, attributes: Vec<Attribute>) -> AttributeArray
{
  attributes.into_iter().fold(
    model, |model, attr| check(cx, model, attr)
  )
}

pub fn check(cx: &ExtCtxt, model: AttributeArray, attr: Attribute) -> AttributeArray
{
  let meta_item = attr.node.value;
  match_meta_item(cx, model, &meta_item)
}

fn match_meta_item(cx: &ExtCtxt,
  model: AttributeArray,
  meta_item: &P<MetaItem>) -> AttributeArray
{
  let meta_name = meta_item_name(meta_item.node.clone());
  let mut attr_exists = false;
  let model = model.into_iter().map(|info|
    if info.name == meta_name {
      attr_exists = true;
      match_model(cx, info, meta_item)
    } else {
      info
    }
  ).collect();
  if !attr_exists {
    unknown_attribute(cx, meta_name, meta_item.span);
  }
  model
}

fn meta_item_name(meta_item: MetaItem_) -> InternedString
{
  match meta_item {
    MetaWord(name) |
    MetaList(name, _) |
    MetaNameValue(name, _) => name
  }
}

fn match_model(cx: &ExtCtxt, info: AttributeInfo, meta_item: &P<MetaItem>) -> AttributeInfo
{
  let model = match (info.model, meta_item.node.clone()) {
    (UnitValue(value), MetaWord(_)) => UnitValue(match_value(cx, value, meta_item.span)),
    (KeyValue(mlit), MetaNameValue(_, lit)) => KeyValue(match_lit(cx, mlit, lit)),
    (SubAttribute(dict), MetaList(_, list)) => SubAttribute(match_sub_attributes(cx, dict, list)),
    (model, _) => model_mismatch(cx, model, meta_item)
  };
  AttributeInfo { name: info.name, desc: info.desc, model: model }
}

fn match_value(cx: &ExtCtxt, value: AttributeValue<()>, span: Span) -> AttributeValue<()>
{
  value.update(cx, (), span)
}

fn match_lit(cx: &ExtCtxt, mlit: AttributeLitModel, lit: Lit) -> AttributeLitModel
{
  let sp = lit.span;
  match (mlit, lit.node.clone()) {
    (MLitStr(value), LitStr(s, style)) => MLitStr(value.update(cx, (s, style), sp)),
    (MLitBinary(value), LitBinary(val)) => MLitBinary(value.update(cx, val, sp)),
    (MLitByte(value), LitByte(val)) => MLitByte(value.update(cx, val, sp)),
    (MLitChar(value), LitChar(val)) => MLitChar(value.update(cx, val, sp)),
    (MLitInt(value), LitInt(val, ty)) => MLitInt(value.update(cx, (val, ty), sp)),
    (MLitFloat(value), LitFloat(val, ty)) => MLitFloat(value.update(cx, (val, ty), sp)),
    (MLitFloatUnsuffixed(value), LitFloatUnsuffixed(val)) => MLitFloatUnsuffixed(value.update(cx, val, sp)),
    (MLitBool(value), LitBool(val)) => MLitBool(value.update(cx, val, sp)),
    (mlit, _) => lit_mismatch(cx, mlit, lit)
  }
}

fn match_sub_attributes(cx: &ExtCtxt, model: AttributeArray, meta_items: Vec<P<MetaItem>>) -> AttributeArray
{
  meta_items.iter().fold(model, |model, meta_item| match_meta_item(cx, model, meta_item))
}

fn model_mismatch(cx: &ExtCtxt, model: AttributeModel, meta_item: &P<MetaItem>) -> AttributeModel
{
  cx.span_err(meta_item.span, "Model mismatch.");
  model
}

fn lit_mismatch(cx: &ExtCtxt, mlit: AttributeLitModel, lit: Lit) -> AttributeLitModel
{
  let mlit_printer = mlit.to_lit_printer();
  let lit_printer = lit_to_lit_printer(&lit.node);
  cx.span_err(lit.span,
    format!("Expected {} literal (e.g. `key = {}`) but got {} literal (e.g. `key = {}`).",
      mlit_printer.type_to_str(), mlit_printer.type_example_to_str(),
      lit_printer.type_to_str(), lit_printer.type_example_to_str()).as_str());
  mlit
}

fn unknown_attribute(cx: &ExtCtxt, _meta_name: InternedString, span: Span)
{
  cx.span_err(span, "Unknown attribute.");
  // model.doc_approaching_results(cx, context, meta_name, span);
}
