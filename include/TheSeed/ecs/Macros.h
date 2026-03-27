#ifndef TS_ECS_MACROS_H
#define TS_ECS_MACROS_H

#include <TheSeed/ecs/Scene.h>
#include <string.h>

// Getter and Setter Macros

#define TS_BASIC_GETTER(component_type, field_name, field_exp, field_size)     \
  void *ts_get_##component_type##_##field_name(const void *ptr) {              \
    const component_type *component = ptr;                                     \
    void *data = malloc(field_size);                                           \
    memcpy(data, field_exp, field_size);                                       \
    return data;                                                               \
  }

#define TS_BASIC_SETTER(component_type, field_name, field_exp, field_size)     \
  void ts_set_##component_type##_##field_name(void *ptr, const void *data) {   \
    component_type *component = ptr;                                           \
    memcpy(field_exp, data, field_size);                                       \
  }

#define TS_STRING_GETTER(component_type, field_name, field_exp)                \
  void *ts_get_##component_type##_##field_name(const void *ptr) {              \
    const component_type *component = ptr;                                     \
    return ts_copy_char_ptr(field_exp);                                        \
  }

#define TS_STRING_SETTER(component_type, field_name, field_exp)                \
  void ts_set_##component_type##_##field_name(void *ptr, const void *data) {   \
    component_type *component = ptr;                                           \
    if (field_exp) {                                                           \
      free(field_exp);                                                         \
    }                                                                          \
    field_exp = ts_copy_char_ptr(data);                                        \
  }

// Serialization and Deserialization Macros

// NOLINTBEGIN(bugprone-macro-parentheses)
#define TS_BASIC_SERIALIZE(component_type, field_name, field_exp, field_size)  \
  char *ts_serialize_##component_type##_##field_name(const void *ptr) {        \
    const component_type *component = ptr;                                     \
    char *field_id = #field_name;                                              \
                                                                               \
    size_t allocation = sizeof(size_t) + strlen(field_id) + 1 + (field_size);  \
    char *data = (char *)malloc(allocation);                                   \
    char *iter = data;                                                         \
                                                                               \
    memcpy(iter, &allocation, sizeof(size_t));                                 \
    iter += sizeof(size_t);                                                    \
                                                                               \
    memcpy(iter, field_id, strlen(field_id) + 1);                              \
    iter += strlen(field_id) + 1;                                              \
                                                                               \
    memcpy(iter, field_exp, field_size);                                       \
    iter += (field_size);                                                      \
                                                                               \
    return data;                                                               \
  }

#define TS_BASIC_DESERIALIZE(component_type, field_name, field_exp,            \
                             field_size)                                       \
  int ts_deserialize_##component_type##_##field_name(void *ptr,                \
                                                     const char *data) {       \
    component_type *component = ptr;                                           \
    memcpy(field_exp, data, field_size);                                       \
    return 0;                                                                  \
  }

#define TS_BASIC_SERIALIZERS(component_type, field_name, field_exp,            \
                             field_size)                                       \
  TS_BASIC_SERIALIZE(component_type, field_name, field_exp, field_size);       \
  TS_BASIC_DESERIALIZE(component_type, field_name, field_exp, field_size)

#define TS_STRING_SERIALIZE(component_type, field_name, field_exp)             \
  TS_BASIC_SERIALIZE(component_type, field_name, field_exp,                    \
                     strlen(field_exp) + 1)

#define TS_STRING_DESERIALIZE(component_type, field_name, field_exp)           \
  int ts_deserialize_##component_type##_##field_name(void *ptr,                \
                                                     const char *data) {       \
    component_type *component = ptr;                                           \
    if (field_exp) {                                                           \
      free(field_exp);                                                         \
    }                                                                          \
    field_exp = ts_copy_char_ptr(data);                                        \
    return 0;                                                                  \
  }

#define TS_STRING_SERIALIZERS(component_type, field_name, field_exp)           \
  TS_STRING_SERIALIZE(component_type, field_name, field_exp);                  \
  TS_STRING_DESERIALIZE(component_type, field_name, field_exp)

#define TS_SET_DESERIALIZE(component_type, field_name, field_exp, setter)      \
  int ts_deserialize_##component_type##_##field_name(void *ptr,                \
                                                     const char *data) {       \
    setter(ptr, (void *)data);                                                 \
    return 0;                                                                  \
  }
// NOLINTEND(bugprone-macro-parentheses)

#endif
