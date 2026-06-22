/**
 * src/lib/api/customFields.ts — Tauri IPC wrappers for custom field foundation APIs.
 */

import { invoke } from '@tauri-apps/api/core';

export type CustomFieldEntityType = 'contact' | 'deal' | 'activity';
export type CustomFieldType = 'text' | 'number' | 'date' | 'boolean' | 'select';

export interface CustomFieldDefinition {
  id: string;
  entityType: CustomFieldEntityType;
  fieldName: string;
  fieldType: CustomFieldType;
  fieldOptions: string | null;
  sortOrder: number;
  createdAt: string;
}

export interface CustomFieldValue {
  id: string;
  fieldDefId: string;
  entityId: string;
  value: string;
  createdAt: string;
  updatedAt: string;
}

export interface EntityCustomFieldValue {
  valueId: string;
  fieldDefId: string;
  fieldName: string;
  fieldType: CustomFieldType;
  fieldOptions: string | null;
  sortOrder: number;
  value: string;
  updatedAt: string;
}

interface BackendCustomFieldDefinition {
  id: string;
  entity_type: CustomFieldEntityType;
  field_name: string;
  field_type: CustomFieldType;
  field_options: string | null;
  sort_order: number;
  created_at: string;
}

interface BackendCustomFieldValue {
  id: string;
  field_def_id: string;
  entity_id: string;
  value: string;
  created_at: string;
  updated_at: string;
}

interface BackendEntityCustomFieldValue {
  value_id: string;
  field_def_id: string;
  field_name: string;
  field_type: CustomFieldType;
  field_options: string | null;
  sort_order: number;
  value: string;
  updated_at: string;
}

function mapDefinition(definition: BackendCustomFieldDefinition): CustomFieldDefinition {
  return {
    id: definition.id,
    entityType: definition.entity_type,
    fieldName: definition.field_name,
    fieldType: definition.field_type,
    fieldOptions: definition.field_options,
    sortOrder: definition.sort_order,
    createdAt: definition.created_at,
  };
}

function mapValue(value: BackendCustomFieldValue): CustomFieldValue {
  return {
    id: value.id,
    fieldDefId: value.field_def_id,
    entityId: value.entity_id,
    value: value.value,
    createdAt: value.created_at,
    updatedAt: value.updated_at,
  };
}

function mapEntityValue(value: BackendEntityCustomFieldValue): EntityCustomFieldValue {
  return {
    valueId: value.value_id,
    fieldDefId: value.field_def_id,
    fieldName: value.field_name,
    fieldType: value.field_type,
    fieldOptions: value.field_options,
    sortOrder: value.sort_order,
    value: value.value,
    updatedAt: value.updated_at,
  };
}

export async function listCustomFieldDefinitions(
  entityType?: CustomFieldEntityType,
): Promise<CustomFieldDefinition[]> {
  const definitions = await invoke<BackendCustomFieldDefinition[]>('list_custom_field_defs', {
    entity_type: entityType,
  });
  return definitions.map(mapDefinition);
}

export async function createCustomFieldDefinition(payload: {
  entityType: CustomFieldEntityType;
  fieldName: string;
  fieldType: CustomFieldType;
  fieldOptions?: string | null;
  sortOrder?: number;
}): Promise<CustomFieldDefinition> {
  const definition = await invoke<BackendCustomFieldDefinition>('create_custom_field_def', {
    entity_type: payload.entityType,
    field_name: payload.fieldName,
    field_type: payload.fieldType,
    field_options: payload.fieldOptions ?? null,
    sort_order: payload.sortOrder ?? 0,
  });
  return mapDefinition(definition);
}

export async function updateCustomFieldDefinition(payload: {
  id: string;
  fieldName?: string;
  fieldType?: CustomFieldType;
  fieldOptions?: string | null;
  sortOrder?: number;
}): Promise<CustomFieldDefinition> {
  const definition = await invoke<BackendCustomFieldDefinition>('update_custom_field_def', {
    id: payload.id,
    field_name: payload.fieldName,
    field_type: payload.fieldType,
    field_options: payload.fieldOptions,
    sort_order: payload.sortOrder,
  });
  return mapDefinition(definition);
}

export async function deleteCustomFieldDefinition(id: string): Promise<void> {
  await invoke<void>('delete_custom_field_def', { id });
}

export async function setCustomFieldValue(payload: {
  fieldDefId: string;
  entityId: string;
  value: string;
}): Promise<CustomFieldValue> {
  const value = await invoke<BackendCustomFieldValue>('set_custom_field_value', {
    field_def_id: payload.fieldDefId,
    entity_id: payload.entityId,
    value: payload.value,
  });
  return mapValue(value);
}

export async function listCustomFieldValues(
  entityType: CustomFieldEntityType,
  entityId: string,
): Promise<EntityCustomFieldValue[]> {
  const values = await invoke<BackendEntityCustomFieldValue[]>('list_custom_field_values', {
    entity_type: entityType,
    entity_id: entityId,
  });
  return values.map(mapEntityValue);
}
