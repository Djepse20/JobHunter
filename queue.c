#include <stddef.h>
#include <stdlib.h>
#define DefQueue(type)                                                         \
  typedef struct Node##type {                                                  \
    type val;                                                                  \
    struct Node##type *next;                                                   \
  } Node##type;                                                                \
  typedef struct Queue##type {                                                 \
                                                                               \
    Node##type *head;                                                          \
    Node##type *end;                                                           \
    size_t size;                                                               \
  } Queue##type;                                                               \
  Queue##type new_queue_##type() {                                             \
    Queue##type queue;                                                         \
    queue.size = 0;                                                            \
    queue.head = NULL;                                                         \
    return queue;                                                              \
  }                                                                            \
  void queue_push_##type(Queue##type *s, type val) {                           \
    Node##type *node = malloc(sizeof(Node##type));                             \
    if (node == NULL) {                                                        \
      return;                                                                  \
    }                                                                          \
    node->val = val;                                                           \
    node->next = NULL;                                                         \
    s->size++;                                                                 \
    if (s->head == NULL) {                                                     \
      s->head = node;                                                          \
      s->end = node;                                                           \
      return;                                                                  \
    }                                                                          \
    s->end->next = node;                                                       \
    s->end = s->end->next;                                                     \
  }                                                                            \
  type queue_pop_##type(Queue##type *s) {                                      \
    type val = s->head->val;                                                   \
    Node##type *tmp = s->head;                                                 \
    s->head = tmp->next;                                                       \
    free(tmp);                                                                 \
    s->size--;                                                                 \
    return val;                                                                \
  }

#define Queue(type) Queue##type
#define queue_push(type, s, val) queue_push_##type(s, val)
#define queue_pop(type, s) queue_pop_##type(s)
#define new_queue(type) new_queue_##type()
DefQueue(double);
DefQueue(int);