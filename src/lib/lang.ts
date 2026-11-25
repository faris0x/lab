// Copyright (c) 2026 Faris Alfarhan
// SPDX-License-Identifier: GPL-3.0-only

import { readable } from 'svelte/store';

type Lang = 'en' | 'ar';

/**
 * Arabic is intentionally frozen for now. Keeping the store means the
 * remaining `data-ar` markup and dormant translations stay in place without
 * any chance of a stale language event switching a tool into Arabic.
 */
export const lang = readable<Lang>('en');
