import { describe, expect, it } from 'vitest';

import {
  getImportFieldOptions,
  suggestImportMapping,
  toBackendMapping,
  validateImportMapping,
} from './importWizard';

describe('import wizard helpers', () => {
  const contactCustomFields = [
    {
      id: 'field-vip-tier',
      entity_type: 'contact' as const,
      field_name: 'VIP Tier',
      field_type: 'text' as const,
      field_options: null,
      sort_order: 0,
      created_at: '2026-06-25T00:00:00Z',
    },
  ];
  const duplicateContactCustomFields = [
    ...contactCustomFields,
    {
      id: 'field-vip-tier-secondary',
      entity_type: 'contact' as const,
      field_name: 'VIP Tier',
      field_type: 'text' as const,
      field_options: null,
      sort_order: 1,
      created_at: '2026-06-25T00:00:00Z',
    },
  ];
  const activityCustomFields = [
    {
      id: 'field-outcome',
      entity_type: 'activity' as const,
      field_name: 'Outcome',
      field_type: 'text' as const,
      field_options: null,
      sort_order: 0,
      created_at: '2026-06-25T00:00:00Z',
    },
  ];
  const organizationCustomFields = [
    {
      id: 'field-segment',
      entity_type: 'organization' as const,
      field_name: 'Segment',
      field_type: 'text' as const,
      field_options: null,
      sort_order: 0,
      created_at: '2026-06-25T00:00:00Z',
    },
  ];

  it('suggests contact mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('contacts', ['Given Name', 'Surname', 'Company', 'Email Address', 'Ignore']),
    ).toEqual({
      'Given Name': 'first_name',
      Surname: 'last_name',
      Company: 'org_name',
      'Email Address': 'email',
      Ignore: null,
    });
  });

  it('suggests organization mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('organizations', [
        'Company Name',
        'Telephone',
        'URL',
        'Zip Code',
        'Notes',
      ]),
    ).toEqual({
      'Company Name': 'name',
      Telephone: 'phone',
      URL: 'website',
      'Zip Code': 'postal_code',
      Notes: 'description',
    });
  });

  it('suggests deal mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('deals', [
        'Opportunity Name',
        'Amount',
        'Pipeline Stage',
        'Close Date',
        'Memo',
      ]),
    ).toEqual({
      'Opportunity Name': 'title',
      Amount: 'value',
      'Pipeline Stage': 'stage',
      'Close Date': 'expected_close',
      Memo: 'notes',
    });
  });

  it('suggests activity mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('activities', [
        'Type',
        'Subject',
        'Details',
        'Due Date',
        'Done',
        'Local Contact ID',
      ]),
    ).toEqual({
      Type: 'activity_type',
      Subject: 'title',
      Details: 'description',
      'Due Date': 'due_date',
      Done: 'completed',
      'Local Contact ID': 'contact_id',
    });
  });

  it('adds contact custom fields as mapping options and suggestions', () => {
    expect(getImportFieldOptions('contacts', contactCustomFields)).toContainEqual({
      value: 'custom:VIP Tier',
      label: 'Custom: VIP Tier',
    });

    expect(
      suggestImportMapping('contacts', ['First Name', 'VIP Tier'], contactCustomFields),
    ).toEqual({
      'First Name': 'first_name',
      'VIP Tier': 'custom:VIP Tier',
    });
  });

  it('adds activity custom fields as mapping options and suggestions', () => {
    expect(getImportFieldOptions('activities', activityCustomFields)).toContainEqual({
      value: 'custom:Outcome',
      label: 'Custom: Outcome',
    });

    expect(
      suggestImportMapping('activities', ['Type', 'Subject', 'Outcome'], activityCustomFields),
    ).toEqual({
      Type: 'activity_type',
      Subject: 'title',
      Outcome: 'custom:Outcome',
    });
  });

  it('accepts supported custom mappings and rejects unsupported ones', () => {
    expect(
      validateImportMapping(
        'contacts',
        {
          'First Name': 'first_name',
          'VIP Tier': 'custom:VIP Tier',
        },
        contactCustomFields,
      ),
    ).toEqual({ valid: true, errors: [] });

    const result = validateImportMapping('contacts', {
      'First Name': 'first_name',
      'Unknown Custom': 'custom:Unknown',
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('"Unknown Custom" maps to an unsupported field.');
  });

  it('uses deterministic custom field targets when names collide', () => {
    expect(getImportFieldOptions('contacts', duplicateContactCustomFields)).toEqual(
      expect.arrayContaining([
        {
          value: 'custom:VIP Tier#field-vip-tier',
          label: 'Custom: VIP Tier (field-vip-tier)',
        },
        {
          value: 'custom:VIP Tier#field-vip-tier-secondary',
          label: 'Custom: VIP Tier (field-vip-tier-secondary)',
        },
      ]),
    );

    expect(
      suggestImportMapping(
        'contacts',
        ['VIP Tier', 'custom:VIP Tier#field-vip-tier'],
        duplicateContactCustomFields,
      ),
    ).toEqual({
      'VIP Tier': null,
      'custom:VIP Tier#field-vip-tier': 'custom:VIP Tier#field-vip-tier',
    });
  });

  it('escapes custom field target delimiters in field names', () => {
    expect(
      getImportFieldOptions('contacts', [
        {
          id: 'field-plan',
          entity_type: 'contact' as const,
          field_name: 'Plan #',
          field_type: 'text' as const,
          field_options: null,
          sort_order: 0,
          created_at: '2026-06-25T00:00:00Z',
        },
      ]),
    ).toContainEqual({
      value: 'custom:Plan %23',
      label: 'Custom: Plan #',
    });
  });

  it('adds organization custom fields as mapping options and suggestions', () => {
    expect(getImportFieldOptions('organizations', organizationCustomFields)).toContainEqual({
      value: 'custom:Segment',
      label: 'Custom: Segment',
    });

    expect(
      suggestImportMapping('organizations', ['Company Name', 'Segment'], organizationCustomFields),
    ).toEqual({
      'Company Name': 'name',
      Segment: 'custom:Segment',
    });
  });

  it('blocks missing required contact fields and duplicate targets', () => {
    const result = validateImportMapping('contacts', {
      Email: 'email',
      'Email 2': 'email',
      Notes: null,
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('First name is required.');
    expect(result.errors).toContain('Email is mapped more than once: Email, Email 2.');
  });

  it('blocks missing required deal title and duplicate deal targets', () => {
    const result = validateImportMapping('deals', {
      Amount: 'value',
      Total: 'value',
      Notes: null,
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('Title is required.');
    expect(result.errors).toContain('Value is mapped more than once: Amount, Total.');
  });

  it('blocks missing required activity fields and duplicate activity targets', () => {
    const result = validateImportMapping('activities', {
      Subject: 'title',
      Summary: 'title',
      Done: 'completed',
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('Activity type is required.');
    expect(result.errors).toContain('Title is mapped more than once: Subject, Summary.');
  });

  it('accepts valid organization mappings with skipped columns', () => {
    expect(
      validateImportMapping('organizations', {
        Company: 'name',
        Email: 'email',
        Ignore: null,
      }),
    ).toEqual({ valid: true, errors: [] });
  });

  it('normalizes blank UI targets to null for backend mappings', () => {
    expect(
      toBackendMapping({
        Company: 'name',
        Ignore: '',
      }),
    ).toEqual({
      Company: 'name',
      Ignore: null,
    });
  });
});
