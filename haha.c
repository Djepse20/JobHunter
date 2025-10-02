#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define Option(type) Option##type

typedef enum Variant { Some, None } Variant;
#define OptionDef(type)                                                        \
  typedef struct Option(type) {                                                \
    Variant variant;                                                           \
    union Data##type {                                                         \
      type val;                                                                \
      char null;                                                               \
    } data;                                                                    \
                                                                               \
  } Option(type);                                                              \
  Option(type) None_##type() {                                                 \
    Option(type) optional;                                                     \
    optional.data.null = '\0';                                                 \
    optional.variant = None;                                                   \
    return optional;                                                           \
  }                                                                            \
  Option(type) Some_##type(type val) {                                         \
    Option(type) optional;                                                     \
    optional.data.val = val;                                                   \
    optional.variant = Some;                                                   \
    return optional;                                                           \
  }

OptionDef(double);
OptionDef(int);
OptionDef(uint64_t);

#define Some(type, val) Some_##type(val)
#define None(type) None_##type()

#define IfSome(type, var, opt)                                                 \
  for (struct OptHelper {                                                      \
         Option(type) new_opt;                                                 \
         char cond;                                                            \
       } __loop_vars___ = {opt, 1};                                            \
       __loop_vars___.new_opt.variant == Some && __loop_vars___.cond;)         \
    for (type var = __loop_vars___.new_opt.data.val; __loop_vars___.cond;      \
         __loop_vars___.cond = 0)

#define IfSomeAnd(type, var, opt, and)                                         \
  for (struct OptHelper {                                                      \
         Option(type) new_opt;                                                 \
         char cond;                                                            \
       } __loop_vars___ = {opt, 1};                                            \
       __loop_vars___.new_opt.variant == Some && __loop_vars___.cond;)         \
    for (type var = __loop_vars___.new_opt.data.val; __loop_vars___.cond;      \
         __loop_vars___.cond = 0)                                              \
      if (and)
#define IfNone(opt) if (opt.variant == None)

#define IfNoneAnd(opt, and) if (opt.variant == None && and)

#define Stack(type) Stack##type

#define DefStack(type)                                                         \
  typedef struct Stack(type) {                                                 \
    type *stack;                                                               \
    size_t size;                                                               \
    size_t cap;                                                                \
  } Stack(type);                                                               \
  Stack(type) new_stack_##type() {                                             \
    Stack##type stack;                                                         \
    stack.size = 0;                                                            \
    stack.cap = 1;                                                             \
    stack.stack = malloc(sizeof(type));                                        \
    return stack;                                                              \
  }                                                                            \
  void stack_push_##type(Stack(type) * s, type val) {                          \
    if (s->cap == s->size) {                                                   \
      size_t new_cap = s->cap * 2;                                             \
      type *new_stack = realloc(s->stack, new_cap * sizeof(type));             \
      if (new_stack == NULL) {                                                 \
        return;                                                                \
      }                                                                        \
      s->cap = new_cap;                                                        \
      s->stack = new_stack;                                                    \
    }                                                                          \
    s->stack[s->size++] = val;                                                 \
  }                                                                            \
  Option(type) stack_pop_##type(Stack##type *s) {                              \
    if (s->size == 0) {                                                        \
      return None(type);                                                       \
    }                                                                          \
    if (s->cap > 1 && s->size == s->cap / 2) {                                 \
      size_t new_cap = s->cap / 2;                                             \
      type *new_stack = realloc(s->stack, new_cap * sizeof(type));             \
      if (new_stack == NULL) {                                                 \
        return None(type);                                                     \
      }                                                                        \
      s->cap = new_cap;                                                        \
      s->stack = new_stack;                                                    \
    }                                                                          \
                                                                               \
    return Some(type, s->stack[--s->size]);                                    \
  }

#define stack_push(type, s, val) stack_push_##type(s, val)
#define stack_pop(type, s) stack_pop_##type(s)
#define new_stack(type) new_stack_##type()
DefStack(double);
DefStack(int);
DefStack(uint64_t);

#define CheckAdd(type, TYPE)
Option(int) checked_add(int a, int b) {

  if (b > 0 && a > (##_MAX - b)) { // would overflow
    return None(int);
  }

  if (b < 0 && a < (INT_MIN - b)) { // would underflow
    return None(int);
  }

  return Some(int, a + b);
}

/* ---------- Main ---------- */
int main(void) {

  Option(int) overflow_val = checked_add(20, INT_MAX);
  IfSome(int, val, overflow_val) { printf("value : %d\n", val); }

  Option(int) ok_val = checked_add(20, 20);
  IfSome(int, val, ok_val) { printf("value : %d\n", val); }

  size_t num_ele = 20;

  Stack(uint64_t) s = new_stack(uint64_t);

  for (size_t idx = 0; idx < num_ele; idx++) {
    stack_push(uint64_t, &s, idx);
  }
  for (size_t idx = 0; idx < num_ele * 2; idx++) {
    Option(uint64_t) opt = stack_pop(uint64_t, &s);
    IfSomeAnd(uint64_t, y, opt, y > 10) {
      printf("size: %lld ", s.size);
      printf("cap: %lld\n", s.cap);
    }

    IfNone(opt) { printf("HAHA %lld\n", idx); }
  }
}
