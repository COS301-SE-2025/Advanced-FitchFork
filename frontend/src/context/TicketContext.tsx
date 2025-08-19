import type { Ticket } from '@/types/modules/assignments/tickets';
import { createContext, useContext } from 'react';

export interface TicketContextValue {
  ticket: Ticket;
}

const TicketContext = createContext<TicketContextValue | null>(null);

export const useTicket = (): TicketContextValue => {
  const context = useContext(TicketContext);
  if (!context) throw new Error('useTicket must be used within a TicketProvider');
  return context;
};

export const TicketProvider = TicketContext.Provider;
