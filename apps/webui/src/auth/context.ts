import { createSignal } from 'solid-js';
import { isBasicAuth } from './config';

export const [isAuthenticated, setIsAuthenticated] = createSignal(isBasicAuth);
