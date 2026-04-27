import type { PendingValidationDto } from './types';

const BASE = '/api';

export async function getPendingValidations(): Promise<PendingValidationDto[]> {
  const res = await fetch(`${BASE}/validations`);
  if (!res.ok) throw new Error(`Failed to fetch validations: ${res.statusText}`);
  return res.json();
}
