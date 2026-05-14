import { invoke } from '@tauri-apps/api/core';
import type { MenuItem } from '../types';

export const listMenuItems = () => invoke<MenuItem[]>('list_menu_items');
export const execMenuItem = (itemId: string) =>
  invoke<void>('exec_menu_item', { itemId });
