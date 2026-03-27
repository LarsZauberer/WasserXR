#ifndef TS_ECS_MACROS_H
#define TS_ECS_MACROS_H

#include <TheSeed/ecs/Scene.h>
#include <string.h>

// NOLINTBEGIN(bugprone-macro-parentheses)

/** @name Getter and Setter Macros
 * Macros for generating component field accessor functions
 * @{
 */

/**
 * Generate a getter function for a basic component field.
 * Creates a function named ts_get_<component_type>_<field_name> that retrieves
 * the field value from a component instance. The getter returns a const pointer
 * to the field data.
 * @param component_type The type name of the component struct
 * @param field_name The name of the field to generate a getter for
 * @param field_exp Expression that evaluates to the field's address
 * @param field_size Size of the field in bytes (used for documentation, not in
 * getter)
 */
#define TS_BASIC_GETTER(component_type, field_name, field_exp, field_size)     \
  const void *ts_get_##component_type##_##field_name(const void *ptr) {        \
    const component_type *component = ptr;                                     \
    return field_exp;                                                          \
  }

/**
 * Generate a setter function for a basic component field.
 * Creates a function named ts_set_<component_type>_<field_name> that updates
 * the field value on a component instance. Uses memcpy to copy the data.
 * @param component_type The type name of the component struct
 * @param field_name The name of the field to generate a setter for
 * @param field_exp Expression that evaluates to the field's address
 * @param field_size Size of the field in bytes (used by memcpy)
 */
#define TS_BASIC_SETTER(component_type, field_name, field_exp, field_size)     \
  void ts_set_##component_type##_##field_name(void *ptr, const void *data) {   \
    component_type *component = ptr;                                           \
    if (data) {                                                                \
      memcpy(field_exp, data, field_size);                                     \
    }                                                                          \
  }

/**
 * Generate both getter and setter functions for a basic component field.
 * This is a convenience macro that combines TS_BASIC_GETTER and
 * TS_BASIC_SETTER.
 * @param component_type The type name of the component struct
 * @param field_name The name of the field to generate accessors for
 * @param field_exp Expression that evaluates to the field's address
 * @param field_size Size of the field in bytes
 */
#define TS_BASIC_ACCESS(component_type, field_name, field_exp, field_size)     \
  TS_BASIC_GETTER(component_type, field_name, field_exp, field_size);          \
  TS_BASIC_SETTER(component_type, field_name, field_exp, field_size)

/**
 * Generate a getter function for a string component field.
 * Creates a function named ts_get_<component_type>_<field_name> that retrieves
 * a string field value. This is a specialized version of TS_BASIC_GETTER for
 * string types (char*).
 * @param component_type The type name of the component struct
 * @param field_name The name of the string field to generate a getter for
 * @param field_exp Expression that evaluates to the string field's address
 */
#define TS_STRING_GETTER(component_type, field_name, field_exp)                \
  TS_BASIC_GETTER(component_type, field_name, field_exp, 0)

/**
 * Generate a setter function for a string component field.
 * Creates a function named ts_set_<component_type>_<field_name> that updates
 * a string field value. Handles memory management by freeing the old string
 * and allocating memory for the new string using ts_copy_char_ptr.
 * @param component_type The type name of the component struct
 * @param field_name The name of the string field to generate a setter for
 * @param field_exp Expression that evaluates to the string field's address
 */
#define TS_STRING_SETTER(component_type, field_name, field_exp)                \
  void ts_set_##component_type##_##field_name(void *ptr, const void *data) {   \
    component_type *component = ptr;                                           \
    if (field_exp) {                                                           \
      free(field_exp);                                                         \
    }                                                                          \
    field_exp = ts_copy_char_ptr(data);                                        \
  }

/**
 * Generate both getter and setter functions for a string component field.
 * This is a convenience macro that combines TS_STRING_GETTER and
 * TS_STRING_SETTER for string fields that require special memory management.
 * @param component_type The type name of the component struct
 * @param field_name The name of the string field to generate accessors for
 * @param field_exp Expression that evaluates to the string field's address
 */
#define TS_STRING_ACCESS(component_type, field_name, field_exp)                \
  TS_STRING_GETTER(component_type, field_name, field_exp);                     \
  TS_STRING_SETTER(component_type, field_name, field_exp)

/** @} */

/** @name Serialization and Deserialization Macros
 * Macros for generating component field serialization functions
 * @{
 */

/**
 * Generate a serialization function for a basic component field.
 * Creates a function named ts_serialize_<component_type>_<field_name> that
 * serializes a field into a byte stream. The output format is:
 * [size_t: total allocation size][field_name string][field data]
 * @param component_type The type name of the component struct
 * @param field_name The name of the field to generate a serializer for
 * @param field_exp Expression that evaluates to the field's address
 * @param field_size Size of the field in bytes
 * @return Dynamically allocated byte array (caller must free)
 */
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

/**
 * Generate a deserialization function for a basic component field.
 * Creates a function named ts_deserialize_<component_type>_<field_name> that
 * deserializes a field from a byte stream using memcpy.
 * @param component_type The type name of the component struct
 * @param field_name The name of the field to generate a deserializer for
 * @param field_exp Expression that evaluates to the field's address
 * @param field_size Size of the field in bytes
 * @return 0 on success, non-zero on failure
 */
#define TS_BASIC_DESERIALIZE(component_type, field_name, field_exp,            \
                             field_size)                                       \
  int ts_deserialize_##component_type##_##field_name(void *ptr,                \
                                                     const char *data) {       \
    component_type *component = ptr;                                           \
    memcpy(field_exp, data, field_size);                                       \
    return 0;                                                                  \
  }

/**
 * Generate both serialization and deserialization functions for a basic field.
 * This is a convenience macro that combines TS_BASIC_SERIALIZE and
 * TS_BASIC_DESERIALIZE for non-string fields.
 * @param component_type The type name of the component struct
 * @param field_name The name of the field to generate serializers for
 * @param field_exp Expression that evaluates to the field's address
 * @param field_size Size of the field in bytes
 */
#define TS_BASIC_SERIALIZERS(component_type, field_name, field_exp,            \
                             field_size)                                       \
  TS_BASIC_SERIALIZE(component_type, field_name, field_exp, field_size);       \
  TS_BASIC_DESERIALIZE(component_type, field_name, field_exp, field_size)

/**
 * Generate a serialization function for a string component field.
 * Creates a function named ts_serialize_<component_type>_<field_name> that
 * serializes a null-terminated string field. Automatically calculates the
 * string length using strlen.
 * @param component_type The type name of the component struct
 * @param field_name The name of the string field to generate a serializer for
 * @param field_exp Expression that evaluates to the string field's address
 * @return Dynamically allocated byte array (caller must free)
 */
#define TS_STRING_SERIALIZE(component_type, field_name, field_exp)             \
  TS_BASIC_SERIALIZE(component_type, field_name, field_exp,                    \
                     strlen(field_exp) + 1)

/**
 * Generate a deserialization function for a string component field.
 * Creates a function named ts_deserialize_<component_type>_<field_name> that
 * deserializes a string field. Handles memory management by freeing the old
 * string and allocating memory for the new string using ts_copy_char_ptr.
 * @param component_type The type name of the component struct
 * @param field_name The name of the string field to generate a deserializer for
 * @param field_exp Expression that evaluates to the string field's address
 * @return 0 on success, non-zero on failure
 */
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

/**
 * Generate both serialization and deserialization functions for a string field.
 * This is a convenience macro that combines TS_STRING_SERIALIZE and
 * TS_STRING_DESERIALIZE for string fields that require special memory
 * management.
 * @param component_type The type name of the component struct
 * @param field_name The name of the string field to generate serializers for
 * @param field_exp Expression that evaluates to the string field's address
 */
#define TS_STRING_SERIALIZERS(component_type, field_name, field_exp)           \
  TS_STRING_SERIALIZE(component_type, field_name, field_exp);                  \
  TS_STRING_DESERIALIZE(component_type, field_name, field_exp)

/**
 * Generate a deserialization function that uses a custom setter function.
 * Creates a function named ts_deserialize_<component_type>_<field_name> that
 * deserializes a field by delegating to a custom setter function instead of
 * using memcpy. Useful for fields with complex deserialization logic.
 * @param component_type The type name of the component struct
 * @param field_name The name of the field to generate a deserializer for
 * @param field_exp Expression that evaluates to the field's address (unused in
 * implementation)
 * @param setter Custom setter function to use for deserialization
 * @return 0 on success, non-zero on failure
 */
#define TS_SET_DESERIALIZE(component_type, field_name, field_exp, setter)      \
  int ts_deserialize_##component_type##_##field_name(void *ptr,                \
                                                     const char *data) {       \
    setter(ptr, (void *)data);                                                 \
    return 0;                                                                  \
  }

/** @} */

/** @name Schema Generation Macros
 * Macros for registering component fields in schemas
 * @{
 */

/**
 * Register a component field in a schema with custom functions.
 * Creates a TS_Component_Field and adds it to the provided schema. This is
 * the most flexible field registration macro, allowing you to specify all
 * field properties including type, getter, setter, serializer, and
 * deserializer.
 * @param type Primitive type of the field (TS_L, TS_F, TS_C, TS_BLOB, TS_S,
 * TS_BLOB_ARRAY)
 * @param name Name of the field (will be stringified)
 * @param getter Function pointer to get the field value (TS_Component_Getter)
 * @param setter Function pointer to set the field value (TS_Component_Setter)
 * @param serializer Function pointer to serialize the field
 * (TS_Component_Serializer)
 * @param deserializer Function pointer to deserialize the field
 * (TS_Component_Deserializer)
 */
#define TS_SCHEMA_FIELD(type, name, getter, setter, serializer, deserializer)  \
  TS_Component_Field *name##_field = ts_create_component_field(                \
      #name, type, getter, setter, serializer, deserializer);                  \
  ts_add_field_to_component_schema(schema, name##_field)

/**
 * Register a fully-featured component field in a schema.
 * Automatically constructs function names based on component type and field
 * name following the naming convention: ts_<action>_<component_type>_<name>.
 * This macro assumes you've already generated getter, setter, serializer, and
 * deserializer functions using the corresponding macros (e.g., TS_BASIC_ACCESS
 * and TS_BASIC_SERIALIZERS).
 * @param component_type The type name of the component struct
 * @param type Primitive type of the field (TS_L, TS_F, TS_C, TS_BLOB, TS_S,
 * TS_BLOB_ARRAY)
 * @param name Name of the field
 */
#define TS_SCHEMA_FIELD_FULL(component_type, type, name)                       \
  TS_SCHEMA_FIELD(type, name, ts_get_##component_type##_##name,                \
                  ts_set_##component_type##_##name,                            \
                  ts_serialize_##component_type##_##name,                      \
                  ts_deserialize_##component_type##_##name)

/**
 * Register a read-only component field in a schema.
 * Automatically constructs the getter function name based on component type and
 * field name. Only provides a getter - setter, serializer, and deserializer are
 * set to NULL, making this field read-only and non-serializable.
 * @param component_type The type name of the component struct
 * @param type Primitive type of the field (TS_L, TS_F, TS_C, TS_BLOB, TS_S,
 * TS_BLOB_ARRAY)
 * @param name Name of the field
 */
#define TS_SCHEMA_FIELD_GET(component_type, type, name)                        \
  TS_SCHEMA_FIELD(type, name, ts_get_##component_type##_##name, NULL, NULL,    \
                  NULL)

/** @} */

// NOLINTEND(bugprone-macro-parentheses)

#endif
