package works.weave.socks.cart.entities;

import jakarta.persistence.*;
import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.Setter;

import java.util.Objects;

@Entity
@Getter
@Setter
@AllArgsConstructor
@Table(name = "cart_items")
public class Item {
    @Id
   // @GeneratedValue(strategy = GenerationType.)
    @Column(name = "cart_id")
    private int customerId;

    @Column(name = "item_id")
    private int itemId;

    @Column(name = "quantity")
    private int quantity;

    @Column(name = "price")
    private float price;

    public Item() {
    }

    public int cartId() {
        return customerId;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;

        Item item = (Item) o;

        return Objects.equals(itemId, item.itemId);
    }
}
