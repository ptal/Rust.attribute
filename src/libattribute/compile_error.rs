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

pub use syntax::ext::base::ExtCtxt;
pub use syntax::codemap::Span;
use compile_error::CompileErrorLevel::*;

#[derive(Clone, Copy)]
pub enum CompileErrorLevel
{
  Silent,
  Warn,
  Error
}

impl CompileErrorLevel
{
  pub fn issue<'a>(&self, cx: &'a ExtCtxt, span: Span, msg: &str)
  {
    match *self {
      Silent => (),
      Warn => cx.span_warn(span, msg),
      Error => cx.span_err(span, msg)
    }
  }

  pub fn is_silent(&self) -> bool
  {
    match *self {
      Silent => true,
      _ => false
    }
  }

  pub fn is_error(&self) -> bool
  {
    match *self {
      Error => true,
      _ => false
    }
  }
}

#[derive(Clone, Copy)]
pub struct DuplicateAttribute
{
  level: CompileErrorLevel,
  extra_msg: Option<&'static str>
}

impl DuplicateAttribute
{
  pub fn new(level: CompileErrorLevel, extra_msg: Option<&'static str>)
    -> DuplicateAttribute
  {
    DuplicateAttribute {
      level: level,
      extra_msg: extra_msg
    }
  }

  pub fn simple(level: CompileErrorLevel) -> DuplicateAttribute
  {
    DuplicateAttribute::new(level, None)
  }

  pub fn error(extra_msg: &'static str) -> DuplicateAttribute
  {
    DuplicateAttribute::new(Error, Some(extra_msg))
  }

  pub fn issue<'a>(&self, cx: &'a ExtCtxt, span: Span, previous_span: Span) -> bool
  {
    let extra_msg = self.extra_msg.unwrap_or("");
    self.level.issue(cx, span,
      format!("Duplicate attribute. {}", extra_msg).as_slice());
    if !self.level.is_silent() {
      cx.span_note(previous_span,
        "Previous declaration here.");
    }
    self.level.is_error()
  }
}
