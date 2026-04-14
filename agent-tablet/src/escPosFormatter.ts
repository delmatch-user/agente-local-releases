// Formata o conteúdo do MiaCardapio para comandos ESC/POS básicos
export class EscPosFormatter {
  constructor(private width: number = 32) {} // 32 chars for 58mm, 42-48 for 80mm

  formatReceipt(content: any, title: string = "RECIBO"): string {
    let lines: string[] = [];
    
    // Header
    lines.push(this.center("Miacardapio"));
    lines.push(this.center(title.toUpperCase()));
    lines.push("-".repeat(this.width));

    // Order Info
    if (content.order_number) {
      lines.push(`PEDIDO: #${content.order_number}`);
    }
    if (content.customer_name) {
      lines.push(`CLIENTE: ${content.customer_name}`);
    }
    lines.push(`DATA: ${new Date().toLocaleString('pt-BR')}`);
    lines.push("-".repeat(this.width));

    // Items
    if (content.items && Array.isArray(content.items)) {
      content.items.forEach((item: any) => {
        const qty = item.quantity || 1;
        const name = item.name || "Item";
        const price = item.price ? `R$ ${item.price.toFixed(2)}` : "";
        
        // Formato: "1x Nome do Item"
        lines.push(`${qty}x ${name}`);
        if (price) {
          lines.push(this.rightAlign(price));
        }
        
        // Observações
        if (item.notes) {
          lines.push(`  obs: ${item.notes}`);
        }
      });
    }

    lines.push("-".repeat(this.width));

    // Totals
    if (content.total) {
      lines.push(this.spaceBetween("TOTAL:", `R$ ${content.total.toFixed(2)}`));
    }

    lines.push("\n\n"); // Espaço extra para corte
    return lines.join("\n");
  }

  private center(text: string): string {
    const pad = Math.max(0, Math.floor((this.width - text.length) / 2));
    return " ".repeat(pad) + text;
  }

  private spaceBetween(left: string, right: string): string {
    const spaces = Math.max(1, this.width - left.length - right.length);
    return left + " ".repeat(spaces) + right;
  }

  private rightAlign(text: string): string {
    const pad = Math.max(0, this.width - text.length);
    return " ".repeat(pad) + text;
  }
}
