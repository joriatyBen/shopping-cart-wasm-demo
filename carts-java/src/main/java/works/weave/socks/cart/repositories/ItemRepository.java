package works.weave.socks.cart.repositories;

import jakarta.transaction.Transactional;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Modifying;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.data.rest.core.annotation.RepositoryRestResource;
import works.weave.socks.cart.entities.Item;

import java.util.List;

@RepositoryRestResource
@Transactional
public interface ItemRepository extends JpaRepository<Item, String> {
  Item findByItemId(@Param("itemId") int itemId);

  List<Item> findByCustomerId(@Param("customerId") int customerId);

  @Modifying
  @Query("UPDATE Item i SET i.quantity = ?3, i.price = ?4 WHERE i.customerId = ?1 AND i.itemId = ?2")
  void updateAllByCustomerId(int customerId, int itemId, int quantity, float price);

  @Modifying
  @Query("DELETE FROM Item i where i.customerId = ?1 AND i.itemId = ?2")
  void deleteByCustomerIdAndItemId(int customerId, int itemId);

  @Modifying
  @Query("DELETE FROM Item i where i.customerId = ?1")
  void deleteByCustomerId(int customerId);
}
