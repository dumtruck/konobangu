import type { Generated, Insertable, Selectable, Updateable } from 'kysely';

export interface PageTable {
  id: Generated<number>;
  name: string | null;
}

export type Persion = Selectable<PageTable>;
export type PersionNew = Insertable<PageTable>;
export type PersionUpdate = Updateable<PageTable>;
