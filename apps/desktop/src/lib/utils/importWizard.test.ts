import { describe, expect, it } from 'vitest';

import {
  suggestImportMapping,
  toBackendMapping,
  validateImportMapping,
} from './importWizard';

describe('import wizard helpers', () => {
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
